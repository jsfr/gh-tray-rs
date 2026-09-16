use crate::types::*;
use muda::{Menu, MenuId, MenuItem, PredefinedMenuItem};
use std::collections::HashMap;
use tray_icon::Icon;

/// Actions that can be triggered from menu items.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    OpenUrl(String),
    CopyUrl(String),
    ToggleAutoStart,
    Quit,
}

/// Shift-click on a PR row copies its link instead of opening it. Every other
/// row keeps its action.
pub fn apply_modifier(action: MenuAction, shift_held: bool) -> MenuAction {
    match action {
        MenuAction::OpenUrl(url) if shift_held => MenuAction::CopyUrl(url),
        other => other,
    }
}

/// Get the emoji prefix for a PR's status (priority order matches F# version).
pub fn status_prefix(pr: &PullRequest) -> &'static str {
    match (
        pr.is_draft,
        pr.has_conflicts,
        &pr.check_status,
        &pr.review_status,
    ) {
        (true, _, _, _) => "\u{1F6A7} ",                      // 🚧 Draft
        (_, true, _, _) => "\u{2694}\u{FE0F} ",               // ⚔️ Conflicts
        (_, _, Some(CheckStatus::Failure), _) => "\u{274C} ", // ❌ Check failure
        (_, _, _, Some(ReviewStatus::ChangesRequested)) => "\u{1F44E} ", // 👎 Changes requested
        (_, _, Some(CheckStatus::Pending), _) => "\u{23F3} ", // ⏳ Check pending
        (_, _, _, Some(ReviewStatus::Approved)) => "\u{1F44D} ", // 👍 Approved
        (_, _, Some(CheckStatus::Success), _) => "\u{2705} ", // ✅ Check success
        _ => "",
    }
}

/// Emoji prefix for the Review group: only viewer's review state matters.
pub fn review_prefix(pr: &PullRequest) -> &'static str {
    match &pr.viewer_review_state {
        Some(ViewerReviewState::Approved) => "\u{1F44D} ", // 👍 Approved
        Some(ViewerReviewState::ChangesRequested) => "\u{1F44E} ", // 👎 Changes requested
        Some(ViewerReviewState::Commented) => "\u{1F4AC} ", // 💬 Commented
        None => "\u{1F440} ",                              // 👀 Review requested
    }
}

/// Extract repo name from "owner/repo" format.
fn repo_name(name_with_owner: &str) -> &str {
    name_with_owner
        .split_once('/')
        .map_or(name_with_owner, |(_, name)| name)
}

/// Width and height of the tray icon in pixels.
const ICON_SIZE: u32 = 32;

/// Render a count as a 32x32 RGBA icon (number on colored circle).
pub fn render_icon(text: &str, is_dark: bool) -> Icon {
    Icon::from_rgba(render_icon_rgba(text, is_dark), ICON_SIZE, ICON_SIZE)
        .expect("Failed to create icon")
}

/// The pixel work behind [`render_icon`], kept separate so it can be tested
/// without building a real tray icon.
fn render_icon_rgba(text: &str, is_dark: bool) -> Vec<u8> {
    use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};

    let size = ICON_SIZE;
    let (bg, fg) = if is_dark {
        ([255, 255, 255, 255], [30, 30, 30])
    } else {
        ([60, 60, 60, 255], [255, 255, 255])
    };

    let mut pixels = vec![0u8; (size * size * 4) as usize];

    // Draw filled circle
    let center = (size / 2) as f32;
    let radius = (size / 2 - 1) as f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                let i = ((y * size + x) * 4) as usize;
                pixels[i..i + 4].copy_from_slice(&bg);
            }
        }
    }

    // Draw text centered on the circle using font metrics
    let font_data = include_bytes!("../assets/Inter-Bold.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Failed to load embedded font");

    let scale = if text.len() > 2 { 18.0 } else { 24.0 };
    let px_scale = PxScale::from(scale);
    let scaled_font = font.as_scaled(px_scale);

    // Measure actual text width using glyph advances
    let text_width: f32 = text
        .chars()
        .map(|c| {
            let glyph_id = scaled_font.glyph_id(c);
            scaled_font.h_advance(glyph_id)
        })
        .sum();

    // Vertical centering: the glyph origin sits on the baseline, one ascent
    // below the top of the text line. Digits have no descenders, so the visual
    // height is just the ascent.
    let ascent = scaled_font.ascent();
    let x_offset = ((size as f32 - text_width) / 2.0).round() as i32;
    let y_offset = ((size as f32 - ascent) / 2.0).round() as i32;

    // Walk the baseline, blending each glyph's coverage over the circle
    let mut caret = 0.0;
    for ch in text.chars() {
        let glyph_id = scaled_font.glyph_id(ch);
        let glyph = glyph_id.with_scale_and_position(px_scale, point(caret, ascent));
        caret += scaled_font.h_advance(glyph_id);

        let Some(outlined) = font.outline_glyph(glyph) else {
            continue;
        };
        let bounds = outlined.px_bounds();
        let origin_x = x_offset + bounds.min.x.round() as i32;
        let origin_y = y_offset + bounds.min.y.round() as i32;

        outlined.draw(|glyph_x, glyph_y, coverage| {
            let x = origin_x + glyph_x as i32;
            let y = origin_y + glyph_y as i32;
            if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
                return;
            }
            let i = ((y as u32 * size + x as u32) * 4) as usize;
            blend_over(&mut pixels[i..i + 4], fg, coverage.clamp(0.0, 1.0));
        });
    }

    pixels
}

/// Composite `rgb` over one RGBA pixel with the given coverage, source-over.
fn blend_over(dst: &mut [u8], rgb: [u8; 3], coverage: f32) {
    let dst_alpha = f32::from(dst[3]) / 255.0;
    let out_alpha = coverage + dst_alpha * (1.0 - coverage);
    if out_alpha <= 0.0 {
        return;
    }

    for channel in 0..3 {
        let src = f32::from(rgb[channel]) / 255.0;
        let existing = f32::from(dst[channel]) / 255.0;
        let blended = (src * coverage + existing * dst_alpha * (1.0 - coverage)) / out_alpha;
        dst[channel] = (blended * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    dst[3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// Build the full tray menu from a PR group.
pub fn build_menu(
    group: &PullRequestGroup,
    is_stale: bool,
    last_updated: Option<&str>,
    auto_start_enabled: bool,
) -> (Menu, HashMap<MenuId, MenuAction>) {
    let menu = Menu::new();
    let mut actions = HashMap::new();

    add_section(&menu, &mut actions, "My PRs", &group.mine, status_prefix);
    add_section(
        &menu,
        &mut actions,
        "Assigned",
        &group.assigned,
        status_prefix,
    );
    add_section(
        &menu,
        &mut actions,
        "Review",
        &group.needs_review,
        review_prefix,
    );

    let _ = menu.append(&PredefinedMenuItem::separator());

    // Shift-click is invisible in the menu, so name it
    if group.total_count() > 0 {
        let hint_item = MenuItem::new("Hold \u{21E7} and click to copy a link", false, None);
        let _ = menu.append(&hint_item);
    }

    // Last updated timestamp
    let timestamp_text = match (is_stale, last_updated) {
        (true, Some(ts)) => format!("\u{26A0} Last updated: {ts}"),
        (false, Some(ts)) => format!("Last updated: {ts}"),
        _ => "Last updated: never".to_string(),
    };
    let ts_item = MenuItem::with_id(
        MenuId::new("timestamp"),
        &timestamp_text,
        false, // disabled
        None,
    );
    let _ = menu.append(&ts_item);

    // Auto-start toggle
    let auto_text = if auto_start_enabled {
        "Auto-start: On"
    } else {
        "Auto-start: Off"
    };
    let auto_item = MenuItem::new(auto_text, true, None);
    actions.insert(auto_item.id().clone(), MenuAction::ToggleAutoStart);
    let _ = menu.append(&auto_item);

    let _ = menu.append(&PredefinedMenuItem::separator());

    let quit_item = MenuItem::new("Quit", true, None);
    actions.insert(quit_item.id().clone(), MenuAction::Quit);
    let _ = menu.append(&quit_item);

    (menu, actions)
}

fn add_section(
    menu: &Menu,
    actions: &mut HashMap<MenuId, MenuAction>,
    header: &str,
    prs: &[PullRequest],
    prefix_fn: fn(&PullRequest) -> &'static str,
) {
    if prs.is_empty() {
        return;
    }

    // Section header (disabled menu item, acts as label)
    let header_item = MenuItem::new(header, false, None);
    let _ = menu.append(&header_item);
    let _ = menu.append(&PredefinedMenuItem::separator());

    for pr in prs {
        let prefix = prefix_fn(pr);
        let repo = repo_name(&pr.repository);
        let label = format!("{prefix}{repo}#{} {}", pr.number, pr.title);
        let item = MenuItem::new(&label, true, None);
        actions.insert(item.id().clone(), MenuAction::OpenUrl(pr.url.clone()));
        let _ = menu.append(&item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_prefix_priority_order() {
        // Draft takes priority over everything
        let pr = PullRequest {
            title: "test".into(),
            url: "".into(),
            number: 1,
            repository: "o/r".into(),
            is_draft: true,
            check_status: Some(CheckStatus::Failure),
            review_status: Some(ReviewStatus::ChangesRequested),
            viewer_review_state: None,
            has_conflicts: true,
        };
        assert_eq!(status_prefix(&pr), "\u{1F6A7} ");

        // Conflicts next
        let pr = PullRequest {
            is_draft: false,
            has_conflicts: true,
            check_status: Some(CheckStatus::Failure),
            ..pr
        };
        assert_eq!(status_prefix(&pr), "\u{2694}\u{FE0F} ");

        // Check failure next
        let pr = PullRequest {
            has_conflicts: false,
            check_status: Some(CheckStatus::Failure),
            review_status: Some(ReviewStatus::ChangesRequested),
            ..pr
        };
        assert_eq!(status_prefix(&pr), "\u{274C} ");

        // Changes requested next
        let pr = PullRequest {
            check_status: None,
            review_status: Some(ReviewStatus::ChangesRequested),
            ..pr
        };
        assert_eq!(status_prefix(&pr), "\u{1F44E} ");
    }

    #[test]
    fn status_prefix_success_variants() {
        let base = PullRequest {
            title: "test".into(),
            url: "".into(),
            number: 1,
            repository: "o/r".into(),
            is_draft: false,
            check_status: None,
            review_status: None,
            viewer_review_state: None,
            has_conflicts: false,
        };

        let pr = PullRequest {
            review_status: Some(ReviewStatus::Approved),
            ..base.clone()
        };
        assert_eq!(status_prefix(&pr), "\u{1F44D} ");

        let pr = PullRequest {
            check_status: Some(CheckStatus::Success),
            ..base.clone()
        };
        assert_eq!(status_prefix(&pr), "\u{2705} ");

        assert_eq!(status_prefix(&base), "");
    }

    #[test]
    fn repo_name_extracts_after_slash() {
        assert_eq!(repo_name("owner/repo"), "repo");
        assert_eq!(repo_name("repo"), "repo");
    }

    #[test]
    #[ignore = "muda::Menu requires main thread on macOS"]
    fn build_menu_includes_all_sections() {
        let group = PullRequestGroup {
            mine: vec![make_test_pr(1)],
            assigned: vec![make_test_pr(4)],
            needs_review: vec![make_test_pr(2), make_test_pr(3)],
        };
        let (_, actions) = build_menu(&group, false, Some("12:00:00"), false);
        // 4 PR items + auto-start + quit = 6 actions
        assert_eq!(actions.len(), 6);
    }

    #[test]
    #[ignore = "muda::Menu requires main thread on macOS"]
    fn build_menu_skips_empty_sections() {
        let group = PullRequestGroup {
            mine: vec![make_test_pr(1)],
            assigned: vec![],
            needs_review: vec![],
        };
        let (_, actions) = build_menu(&group, false, Some("12:00:00"), false);
        // 1 PR item + auto-start + quit = 3 actions
        assert_eq!(actions.len(), 3);
    }

    fn make_test_pr(n: u32) -> PullRequest {
        PullRequest {
            title: format!("PR {n}"),
            url: format!("https://example.com/{n}"),
            number: n,
            repository: "org/repo".to_string(),
            is_draft: false,
            check_status: None,
            review_status: None,
            viewer_review_state: None,
            has_conflicts: false,
        }
    }

    fn pixel(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * ICON_SIZE + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    /// Fully covered glyph pixels land within a channel or two of the
    /// foreground colour, depending on how coverage is composited.
    fn is_opaque_fg(p: [u8; 4], fg: [u8; 3]) -> bool {
        p[3] == 255 && (0..3).all(|i| p[i].abs_diff(fg[i]) <= 3)
    }

    const LIGHT_FG: [u8; 3] = [255, 255, 255];

    #[test]
    fn icon_buffer_is_32x32_rgba() {
        let buf = render_icon_rgba("8", false);
        assert_eq!(buf.len() as u32, ICON_SIZE * ICON_SIZE * 4);
    }

    #[test]
    fn icon_corners_stay_transparent() {
        let buf = render_icon_rgba("8", false);
        assert_eq!(pixel(&buf, 0, 0)[3], 0);
        assert_eq!(pixel(&buf, ICON_SIZE - 1, ICON_SIZE - 1)[3], 0);
    }

    #[test]
    fn icon_circle_is_filled_with_the_background_colour() {
        // (2, 16) is inside the circle and clear of the glyphs
        assert_eq!(
            pixel(&render_icon_rgba("8", false), 2, 16),
            [60, 60, 60, 255]
        );
        assert_eq!(
            pixel(&render_icon_rgba("8", true), 2, 16),
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn icon_draws_the_text_in_the_foreground_colour() {
        let buf = render_icon_rgba("8", false);
        let fg_pixels = (0..ICON_SIZE)
            .flat_map(|y| (0..ICON_SIZE).map(move |x| (x, y)))
            .filter(|&(x, y)| is_opaque_fg(pixel(&buf, x, y), LIGHT_FG))
            .count();
        assert!(fg_pixels > 10, "expected glyph pixels, found {fg_pixels}");
    }

    #[test]
    fn icon_text_is_centred() {
        let buf = render_icon_rgba("8", false);
        let lit: Vec<(u32, u32)> = (0..ICON_SIZE)
            .flat_map(|y| (0..ICON_SIZE).map(move |x| (x, y)))
            .filter(|&(x, y)| is_opaque_fg(pixel(&buf, x, y), LIGHT_FG))
            .collect();
        let mid_x = (lit.iter().map(|&(x, _)| x).min().unwrap()
            + lit.iter().map(|&(x, _)| x).max().unwrap())
            / 2;
        let mid_y = (lit.iter().map(|&(_, y)| y).min().unwrap()
            + lit.iter().map(|&(_, y)| y).max().unwrap())
            / 2;
        assert!(mid_x.abs_diff(ICON_SIZE / 2) <= 3, "x centre off: {mid_x}");
        assert!(mid_y.abs_diff(ICON_SIZE / 2) <= 3, "y centre off: {mid_y}");
    }

    #[test]
    fn dark_and_light_icons_differ() {
        assert_ne!(render_icon_rgba("8", true), render_icon_rgba("8", false));
    }

    #[test]
    fn shift_click_on_a_pr_copies_the_url() {
        let action = MenuAction::OpenUrl("https://example.com/1".to_string());
        assert_eq!(
            apply_modifier(action, true),
            MenuAction::CopyUrl("https://example.com/1".to_string())
        );
    }

    #[test]
    fn plain_click_on_a_pr_opens_the_url() {
        let action = MenuAction::OpenUrl("https://example.com/1".to_string());
        assert_eq!(
            apply_modifier(action, false),
            MenuAction::OpenUrl("https://example.com/1".to_string())
        );
    }

    #[test]
    fn shift_click_on_quit_still_quits() {
        assert_eq!(apply_modifier(MenuAction::Quit, true), MenuAction::Quit);
    }

    #[test]
    fn shift_click_on_auto_start_still_toggles() {
        assert_eq!(
            apply_modifier(MenuAction::ToggleAutoStart, true),
            MenuAction::ToggleAutoStart
        );
    }

    #[test]
    fn review_prefix_uses_viewer_review_state() {
        let mut pr = make_test_pr(1);
        pr.has_conflicts = true;
        pr.check_status = Some(CheckStatus::Failure);
        pr.review_status = Some(ReviewStatus::Approved);

        pr.viewer_review_state = Some(ViewerReviewState::Approved);
        assert_eq!(review_prefix(&pr), "\u{1F44D} ");

        pr.viewer_review_state = Some(ViewerReviewState::ChangesRequested);
        assert_eq!(review_prefix(&pr), "\u{1F44E} ");

        pr.viewer_review_state = Some(ViewerReviewState::Commented);
        assert_eq!(review_prefix(&pr), "\u{1F4AC} ");

        pr.viewer_review_state = None;
        assert_eq!(review_prefix(&pr), "\u{1F440} ");
    }
}
