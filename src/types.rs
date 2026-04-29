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
    #[expect(dead_code)]
    ReviewRequired,
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
    pub has_conflicts: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PullRequestGroup {
    pub mine: Vec<PullRequest>,
    pub assigned: Vec<PullRequest>,
    pub needs_review: Vec<PullRequest>,
}

impl PullRequestGroup {
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
        };
        assert_eq!(group.total_count(), 5);
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
            has_conflicts: false,
        }
    }
}
