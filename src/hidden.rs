use crate::types::{HideCondition, PullRequest, PullRequestGroup};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Hidden Review PRs by URL, with the conditions each one waits for.
pub type Hidden = HashMap<String, Vec<HideCondition>>;

/// `hidden.json`, next to `config.json`.
pub fn path() -> PathBuf {
    crate::config::config_path().with_file_name("hidden.json")
}

/// Read the hidden PRs. A missing file means none are hidden.
pub fn load(path: &Path) -> Hidden {
    let json = match std::fs::read_to_string(path) {
        Ok(json) => json,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Hidden::new(),
        Err(e) => {
            tracing::error!("Failed to read {}: {e}", path.display());
            return Hidden::new();
        }
    };
    serde_json::from_str(&json).unwrap_or_else(|e| {
        tracing::error!("Failed to parse {}: {e}", path.display());
        Hidden::new()
    })
}

/// Write the hidden PRs. The file is replaced in one step, so a crash
/// mid-write cannot leave it half written.
pub fn save(path: &Path, hidden: &Hidden) {
    let result = (|| -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(hidden)?)?;
        std::fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        tracing::error!("Failed to save {}: {e}", path.display());
    }
}

/// Unhide every PR whose conditions are all met, and return those PRs with
/// their conditions. PRs that left the Review group, such as merged ones, are
/// forgotten without being returned.
pub fn release(
    group: &PullRequestGroup,
    hidden: &mut Hidden,
) -> Vec<(PullRequest, Vec<HideCondition>)> {
    hidden.retain(|url, _| group.needs_review.iter().any(|pr| pr.url == *url));

    let mut released = Vec::new();
    for pr in &group.needs_review {
        let ready = hidden
            .get(&pr.url)
            .is_some_and(|conditions| conditions.iter().all(|c| c.is_met(pr)));
        if ready && let Some(conditions) = hidden.remove(&pr.url) {
            released.push((pr.clone(), conditions));
        }
    }
    released
}

/// Move hidden PRs out of the Review group and into `hidden`.
pub fn split(mut group: PullRequestGroup, hidden: &Hidden) -> PullRequestGroup {
    let (hidden_prs, visible): (Vec<_>, Vec<_>) = std::mem::take(&mut group.needs_review)
        .into_iter()
        .partition(|pr| hidden.contains_key(&pr.url));
    group.needs_review = visible;
    group.hidden = hidden_prs
        .into_iter()
        .map(|pr| {
            let conditions = hidden[&pr.url].clone();
            (pr, conditions)
        })
        .collect();
    group
}

/// Notification text for a PR that came back.
pub fn release_message(pr: &PullRequest, conditions: &[HideCondition]) -> String {
    let met: Vec<&str> = conditions.iter().map(|c| c.label()).collect();
    format!(
        "{}#{} {}\n{}.",
        pr.repository,
        pr.number,
        pr.title,
        met.join(". ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CheckStatus;

    fn make_pr(number: u32) -> PullRequest {
        PullRequest {
            title: format!("PR {number}"),
            url: format!("https://github.com/org/repo/pull/{number}"),
            number,
            repository: "org/repo".to_string(),
            is_draft: false,
            check_status: None,
            review_status: None,
            viewer_review_state: None,
            viewer_review_requested: false,
            has_conflicts: false,
            is_mergeable: false,
            in_merge_queue: false,
            approved_by_other: false,
        }
    }

    fn review_group(prs: Vec<PullRequest>) -> PullRequestGroup {
        PullRequestGroup {
            needs_review: prs,
            ..Default::default()
        }
    }

    fn hide(pr: &PullRequest, conditions: &[HideCondition]) -> (String, Vec<HideCondition>) {
        (pr.url.clone(), conditions.to_vec())
    }

    #[test]
    fn release_waits_until_every_condition_is_met() {
        let mut pr = make_pr(1);
        pr.check_status = Some(CheckStatus::Success);
        let conditions = [HideCondition::ChecksGreen, HideCondition::ApprovedByOther];
        let mut hidden = Hidden::from([hide(&pr, &conditions)]);

        let released = release(&review_group(vec![pr.clone()]), &mut hidden);
        assert!(released.is_empty());
        assert!(hidden.contains_key(&pr.url));

        pr.approved_by_other = true;
        let released = release(&review_group(vec![pr.clone()]), &mut hidden);
        assert_eq!(released.len(), 1);
        assert_eq!(released[0].0.url, pr.url);
        assert_eq!(released[0].1, conditions);
        assert!(hidden.is_empty());
    }

    #[test]
    fn release_forgets_prs_that_left_review_without_returning_them() {
        let gone = make_pr(1);
        let staying = make_pr(2);
        let mut hidden = Hidden::from([
            hide(&gone, &[HideCondition::ChecksGreen]),
            hide(&staying, &[HideCondition::ChecksGreen]),
        ]);

        let released = release(&review_group(vec![staying.clone()]), &mut hidden);
        assert!(released.is_empty());
        assert_eq!(hidden.keys().collect::<Vec<_>>(), [&staying.url]);
    }

    #[test]
    fn split_moves_hidden_prs_out_of_review_and_the_count() {
        let mut hidden_pr = make_pr(2);
        hidden_pr.title = "Hidden".to_string();
        let hidden = Hidden::from([hide(&hidden_pr, &[HideCondition::ReviewRequested])]);
        let group = review_group(vec![make_pr(1), hidden_pr.clone(), make_pr(3)]);

        let group = split(group, &hidden);
        let visible: Vec<u32> = group.needs_review.iter().map(|pr| pr.number).collect();
        assert_eq!(visible, [1, 3]);
        assert_eq!(group.hidden.len(), 1);
        assert_eq!(group.hidden[0].0.title, "Hidden");
        assert_eq!(group.hidden[0].1, [HideCondition::ReviewRequested]);
        assert_eq!(group.total_count(), 2);
    }

    #[test]
    fn save_then_load_keeps_the_conditions() {
        let path = std::env::temp_dir().join(format!("gh-tray-hidden-{}.json", std::process::id()));
        let hidden = Hidden::from([(
            "https://github.com/org/repo/pull/1".to_string(),
            vec![
                HideCondition::ApprovedByOther,
                HideCondition::ConflictsResolved,
            ],
        )]);

        save(&path, &hidden);
        let json = std::fs::read_to_string(&path).unwrap();
        let loaded = load(&path);
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded, hidden);
        assert!(json.contains("\"approvedByOther\""), "file held: {json}");
    }

    #[test]
    fn load_of_a_missing_file_hides_nothing() {
        let path =
            std::env::temp_dir().join(format!("gh-tray-missing-{}.json", std::process::id()));
        std::fs::remove_file(&path).ok();
        assert!(load(&path).is_empty());
    }

    #[test]
    fn release_message_names_the_pr_and_what_happened() {
        let pr = make_pr(12);
        let message = release_message(
            &pr,
            &[HideCondition::ChecksGreen, HideCondition::ReviewRequested],
        );
        assert_eq!(
            message,
            "org/repo#12 PR 12\nCI is green. Your review is re-requested."
        );
    }
}
