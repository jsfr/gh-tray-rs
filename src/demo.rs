use crate::types::*;

pub fn demo_pull_requests() -> PullRequestGroup {
    PullRequestGroup {
        mine: vec![
            make_pr(
                1,
                "demo/app",
                "Add user authentication",
                false,
                Some(CheckStatus::Success),
                Some(ReviewStatus::Approved),
                false,
            ),
            make_pr(
                2,
                "demo/app",
                "WIP: Refactor database layer",
                true,
                None,
                None,
                false,
            ),
            make_pr(
                3,
                "demo/api",
                "Fix pagination endpoint",
                false,
                Some(CheckStatus::Failure),
                None,
                false,
            ),
            make_pr(
                4,
                "demo/api",
                "Update dependencies",
                false,
                None,
                None,
                true,
            ),
            make_pr(
                5,
                "demo/web",
                "Add dark mode support",
                false,
                Some(CheckStatus::Pending),
                None,
                false,
            ),
        ],
        review_requested: vec![
            make_pr(
                10,
                "demo/lib",
                "Improve error handling",
                false,
                None,
                Some(ReviewStatus::ChangesRequested),
                false,
            ),
            make_pr(
                11,
                "demo/lib",
                "Add retry logic",
                false,
                Some(CheckStatus::Success),
                None,
                false,
            ),
            make_pr(
                12,
                "demo/docs",
                "Update API documentation",
                false,
                None,
                None,
                false,
            ),
        ],
        involved: vec![
            make_pr(
                20,
                "demo/infra",
                "Migrate to new CI pipeline",
                false,
                Some(CheckStatus::Failure),
                Some(ReviewStatus::ChangesRequested),
                false,
            ),
            make_pr(
                21,
                "demo/infra",
                "Add monitoring dashboard",
                true,
                Some(CheckStatus::Pending),
                None,
                false,
            ),
        ],
    }
}

fn make_pr(
    number: u32,
    repo: &str,
    title: &str,
    is_draft: bool,
    check_status: Option<CheckStatus>,
    review_status: Option<ReviewStatus>,
    has_conflicts: bool,
) -> PullRequest {
    PullRequest {
        title: title.to_string(),
        url: format!("https://github.com/{repo}/pull/{number}"),
        number,
        repository: repo.to_string(),
        is_draft,
        check_status,
        review_status,
        has_conflicts,
    }
}
