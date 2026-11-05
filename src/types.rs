use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Success,
    Failure,
    Pending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewStatus {
    Approved,
    ChangesRequested,
    Commented,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewerReviewState {
    Approved,
    ChangesRequested,
    Commented,
}

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub title: String,
    pub url: String,
    pub number: u32,
    pub repository: String,
    pub is_draft: bool,
    pub check_status: Option<CheckStatus>,
    pub review_status: Option<ReviewStatus>,
    pub viewer_review_state: Option<ViewerReviewState>,
    pub viewer_review_requested: bool,
    pub has_conflicts: bool,
    /// GitHub reports the PR as mergeable. False while GitHub is still
    /// computing it, which it does after every push.
    pub is_mergeable: bool,
    pub in_merge_queue: bool,
    /// Someone other than the viewer approved in their latest decisive review.
    pub approved_by_other: bool,
}

/// A change a hidden Review PR waits for before it comes back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HideCondition {
    ApprovedByOther,
    ChecksGreen,
    ConflictsResolved,
    ReviewRequested,
}

impl HideCondition {
    pub const ALL: [HideCondition; 4] = [
        HideCondition::ApprovedByOther,
        HideCondition::ChecksGreen,
        HideCondition::ConflictsResolved,
        HideCondition::ReviewRequested,
    ];

    pub fn is_met(self, pr: &PullRequest) -> bool {
        match self {
            HideCondition::ApprovedByOther => pr.approved_by_other,
            HideCondition::ChecksGreen => pr.check_status == Some(CheckStatus::Success),
            // Not `!has_conflicts`: that is also true while GitHub is still
            // computing mergeability.
            HideCondition::ConflictsResolved => pr.is_mergeable,
            HideCondition::ReviewRequested => pr.viewer_review_requested,
        }
    }

    /// The dialog's checkbox label, also used in the notification.
    pub fn label(self) -> &'static str {
        match self {
            HideCondition::ApprovedByOther => "Has an approval from someone else",
            HideCondition::ChecksGreen => "CI is green",
            HideCondition::ConflictsResolved => "Conflicts are resolved",
            HideCondition::ReviewRequested => "Your review is re-requested",
        }
    }

    /// Short form for the hidden PR's menu row.
    pub fn short_label(self) -> &'static str {
        match self {
            HideCondition::ApprovedByOther => "approval",
            HideCondition::ChecksGreen => "green CI",
            HideCondition::ConflictsResolved => "no conflicts",
            HideCondition::ReviewRequested => "re-request",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PullRequestGroup {
    pub mine: Vec<PullRequest>,
    pub assigned: Vec<PullRequest>,
    pub needs_review: Vec<PullRequest>,
    /// Review PRs the viewer hid, with the conditions each one waits for.
    pub hidden: Vec<(PullRequest, Vec<HideCondition>)>,
}

impl PullRequestGroup {
    /// The PRs counted on the tray icon. Hidden PRs do not count.
    pub fn total_count(&self) -> usize {
        self.mine.len() + self.assigned.len() + self.needs_review.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_count_sums_all_groups() {
        let group = PullRequestGroup {
            mine: vec![make_pr(1), make_pr(2)],
            assigned: vec![make_pr(7)],
            needs_review: vec![make_pr(3), make_pr(4)],
            hidden: vec![(make_pr(5), vec![HideCondition::ChecksGreen])],
        };
        assert_eq!(group.total_count(), 5);
    }

    #[test]
    fn each_hide_condition_reads_its_own_field() {
        let met = |pr: &PullRequest| -> Vec<HideCondition> {
            HideCondition::ALL
                .into_iter()
                .filter(|c| c.is_met(pr))
                .collect()
        };
        let mut pr = make_pr(1);
        assert_eq!(met(&pr), []);

        pr.approved_by_other = true;
        assert_eq!(met(&pr), [HideCondition::ApprovedByOther]);

        pr.approved_by_other = false;
        pr.check_status = Some(CheckStatus::Success);
        assert_eq!(met(&pr), [HideCondition::ChecksGreen]);

        pr.check_status = None;
        pr.is_mergeable = true;
        assert_eq!(met(&pr), [HideCondition::ConflictsResolved]);

        pr.is_mergeable = false;
        pr.viewer_review_requested = true;
        assert_eq!(met(&pr), [HideCondition::ReviewRequested]);
    }

    #[test]
    fn pending_or_failing_checks_are_not_green() {
        let mut pr = make_pr(1);
        for status in [CheckStatus::Pending, CheckStatus::Failure] {
            pr.check_status = Some(status);
            assert!(!HideCondition::ChecksGreen.is_met(&pr));
        }
    }

    #[test]
    fn total_count_empty_is_zero() {
        let group = PullRequestGroup::default();
        assert_eq!(group.total_count(), 0);
    }

    fn make_pr(number: u32) -> PullRequest {
        PullRequest {
            title: format!("PR {number}"),
            url: format!("https://github.com/test/repo/pull/{number}"),
            number,
            repository: "test/repo".to_string(),
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
}
