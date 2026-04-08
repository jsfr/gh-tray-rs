use crate::types::*;
use image::{Rgba, RgbaImage};
use muda::{Menu, MenuId, MenuItem, PredefinedMenuItem};
use std::collections::HashMap;
use tray_icon::Icon;

/// Actions that can be triggered from menu items.
#[derive(Debug, Clone)]
pub enum MenuAction {
    OpenUrl(String),
    ToggleAutoStart,
    Quit,
}

/// Get the emoji prefix for a PR's status (priority order matches F# version).
pub fn status_prefix(pr: &PullRequest) -> &'static str {
    match (
        pr.is_draft,
        pr.has_conflicts,
        &pr.check_status,
        &pr.review_status,
    ) {
        (true, _, _, _) => "\u{1F6A7} ",                          // 🚧 Draft
        (_, true, _, _) => "\u{2694}\u{FE0F} ",                   // ⚔️ Conflicts
        (_, _, Some(CheckStatus::Failure), _) => "\u{274C} ",     // ❌ Check failure
        (_, _, _, Some(ReviewStatus::ChangesRequested)) => "\u{1F44E} ", // 👎 Changes requested
        (_, _, Some(CheckStatus::Pending), _) => "\u{23F3} ",     // ⏳ Check pending
        (_, _, _, Some(ReviewStatus::Approved)) => "\u{1F44D} ",  // 👍 Approved
        (_, _, Some(CheckStatus::Success), _) => "\u{2705} ",     // ✅ Check success
        _ => "",
    }
}

/// Extract repo name from "owner/repo" format.
fn repo_name(name_with_owner: &str) -> &str {
    name_with_owner
        .split_once('/')
        .map_or(name_with_owner, |(_, name)| name)
}

/// Render a count as a 32x32 RGBA icon (number on colored circle).
pub fn render_icon(text: &str, is_dark: bool) -> Icon {
    let size = 32u32;
    let mut img = RgbaImage::new(size, size);

    let (bg, fg) = if is_dark {
        (Rgba([255, 255, 255, 255]), Rgba([30, 30, 30, 255]))
    } else {
        (Rgba([60, 60, 60, 255]), Rgba([255, 255, 255, 255]))
    };

    // Draw filled circle
    let center = (size / 2) as f32;
    let radius = (size / 2 - 1) as f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                img.put_pixel(x, y, bg);
            }
        }
    }

    // Draw text centered on the circle
    let font_data = include_bytes!("../assets/Inter-Bold.ttf");
    let font =
        ab_glyph::FontRef::try_from_slice(font_data).expect("Failed to load embedded font");

    let scale = if text.len() > 2 { 14.0 } else { 18.0 };

    // Approximate centering
    let text_width = text.len() as f32 * scale * 0.55;
    let x_offset = ((size as f32 - text_width) / 2.0).max(0.0) as i32;
    let y_offset = ((size as f32 - scale) / 2.0 - 1.0).max(0.0) as i32;

    imageproc::drawing::draw_text_mut(&mut img, fg, x_offset, y_offset, scale, &font, text);

    let rgba = img.into_raw();
    Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
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

    add_section(&menu, &mut actions, "My PRs", &group.mine);
    add_section(
        &menu,
        &mut actions,
        "Review Requested",
        &group.review_requested,
    );
    add_section(&menu, &mut actions, "Involved", &group.involved);

    let _ = menu.append(&PredefinedMenuItem::separator());

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
) {
    if prs.is_empty() {
        return;
    }

    // Section header (disabled menu item, acts as label)
    let header_item = MenuItem::new(header, false, None);
    let _ = menu.append(&header_item);
    let _ = menu.append(&PredefinedMenuItem::separator());

    for pr in prs {
        let prefix = status_prefix(pr);
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
            review_requested: vec![make_test_pr(2)],
            involved: vec![make_test_pr(3)],
        };
        let (_, actions) = build_menu(&group, false, Some("12:00:00"), false);
        // 3 PR items + auto-start + quit = 5 actions
        assert_eq!(actions.len(), 5);
    }

    #[test]
    #[ignore = "muda::Menu requires main thread on macOS"]
    fn build_menu_skips_empty_sections() {
        let group = PullRequestGroup {
            mine: vec![make_test_pr(1)],
            review_requested: vec![],
            involved: vec![],
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
            has_conflicts: false,
        }
    }
}
