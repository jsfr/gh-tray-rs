use crate::types::*;
use std::process::Command;

/// Run `gh` CLI with given args and optional token.
pub fn run_gh(token: Option<&str>, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("gh");
    cmd.args(args);

    if let Some(t) = token {
        cmd.env("GH_TOKEN", t);
    }

    let output = cmd.output().map_err(|e| format!("Failed to start gh: {e}"))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Invalid UTF-8 in gh output: {e}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

pub fn validate_auth(token: Option<&str>) -> Result<(), String> {
    run_gh(token, &["auth", "status"]).map(|_| ())
}

pub fn get_username(token: Option<&str>) -> Result<String, String> {
    run_gh(token, &["api", "user", "--jq", ".login"])
}

pub fn resolve_token(account: &str) -> Result<String, String> {
    run_gh(None, &["auth", "token", "--user", account])
}

const GRAPHQL_QUERY: &str = r#"
query($searchQuery: String!) {
  search(query: $searchQuery, type: ISSUE, first: 100) {
    nodes {
      ... on PullRequest {
        title
        url
        number
        isDraft
        repository { nameWithOwner }
        author { login }
        reviewRequests(first: 10) {
          nodes {
            requestedReviewer {
              ... on User { login }
            }
          }
        }
        reviews(last: 1, states: [APPROVED, CHANGES_REQUESTED]) {
          nodes { state }
        }
        commits(last: 1) {
          nodes {
            commit {
              statusCheckRollup { state }
            }
          }
        }
        mergeable
      }
    }
  }
}
"#;

pub fn fetch_pull_requests(
    token: Option<&str>,
    username: &str,
) -> Result<PullRequestGroup, String> {
    let search_query = format!("sort:updated-desc type:pr state:open involves:{username}");
    let query_arg = format!("query={GRAPHQL_QUERY}");
    let search_arg = format!("searchQuery={search_query}");

    let json = run_gh(
        token,
        &["api", "graphql", "-f", &query_arg, "-f", &search_arg],
    )?;

    parse_response(&json, username)
}

fn parse_response(json: &str, username: &str) -> Result<PullRequestGroup, String> {
    let doc: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse JSON: {e}"))?;

    let nodes = &doc["data"]["search"]["nodes"];
    let nodes = nodes
        .as_array()
        .ok_or("Expected nodes array in response")?;

    let mut mine = Vec::new();
    let mut review_requested = Vec::new();
    let mut involved = Vec::new();

    for node in nodes {
        let Some(pr) = parse_pull_request(node) else {
            continue;
        };

        let author = node["author"]["login"].as_str().unwrap_or("");
        let reviewers = parse_reviewer_logins(node);

        if author.eq_ignore_ascii_case(username) {
            mine.push(pr);
        } else if reviewers
            .iter()
            .any(|r| r.eq_ignore_ascii_case(username))
        {
            review_requested.push(pr);
        } else {
            involved.push(pr);
        }
    }

    Ok(PullRequestGroup {
        mine,
        review_requested,
        involved,
    })
}

fn parse_pull_request(node: &serde_json::Value) -> Option<PullRequest> {
    let title = node["title"].as_str()?;

    Some(PullRequest {
        title: title.to_string(),
        url: node["url"].as_str().unwrap_or("").to_string(),
        number: node["number"].as_u64()? as u32,
        repository: node["repository"]["nameWithOwner"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        is_draft: node["isDraft"].as_bool().unwrap_or(false),
        check_status: parse_check_status(node),
        review_status: parse_review_status(node),
        has_conflicts: node["mergeable"].as_str() == Some("CONFLICTING"),
    })
}

fn parse_check_status(node: &serde_json::Value) -> Option<CheckStatus> {
    let commit_node = node["commits"]["nodes"].as_array()?.first()?;
    let state = commit_node["commit"]["statusCheckRollup"]["state"].as_str()?;

    match state {
        "SUCCESS" => Some(CheckStatus::Success),
        "FAILURE" | "ERROR" => Some(CheckStatus::Failure),
        _ => Some(CheckStatus::Pending),
    }
}

fn parse_review_status(node: &serde_json::Value) -> Option<ReviewStatus> {
    let review_node = node["reviews"]["nodes"].as_array()?.first()?;
    let state = review_node["state"].as_str()?;

    match state {
        "APPROVED" => Some(ReviewStatus::Approved),
        "CHANGES_REQUESTED" => Some(ReviewStatus::ChangesRequested),
        _ => None,
    }
}

fn parse_reviewer_logins(node: &serde_json::Value) -> Vec<String> {
    let Some(nodes) = node["reviewRequests"]["nodes"].as_array() else {
        return Vec::new();
    };

    nodes
        .iter()
        .filter_map(|n| n["requestedReviewer"]["login"].as_str())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_response() -> String {
        r#"{
            "data": {
                "search": {
                    "nodes": [
                        {
                            "title": "Add feature",
                            "url": "https://github.com/org/repo/pull/1",
                            "number": 1,
                            "isDraft": false,
                            "repository": { "nameWithOwner": "org/repo" },
                            "author": { "login": "testuser" },
                            "reviewRequests": { "nodes": [] },
                            "reviews": { "nodes": [{ "state": "APPROVED" }] },
                            "commits": { "nodes": [{ "commit": { "statusCheckRollup": { "state": "SUCCESS" } } }] },
                            "mergeable": "MERGEABLE"
                        },
                        {
                            "title": "Review this",
                            "url": "https://github.com/org/repo/pull/2",
                            "number": 2,
                            "isDraft": true,
                            "repository": { "nameWithOwner": "org/repo" },
                            "author": { "login": "other" },
                            "reviewRequests": { "nodes": [{ "requestedReviewer": { "login": "testuser" } }] },
                            "reviews": { "nodes": [] },
                            "commits": { "nodes": [{ "commit": { "statusCheckRollup": { "state": "PENDING" } } }] },
                            "mergeable": "CONFLICTING"
                        },
                        {
                            "title": "Involved PR",
                            "url": "https://github.com/org/repo/pull/3",
                            "number": 3,
                            "isDraft": false,
                            "repository": { "nameWithOwner": "org/repo" },
                            "author": { "login": "someone" },
                            "reviewRequests": { "nodes": [] },
                            "reviews": { "nodes": [{ "state": "CHANGES_REQUESTED" }] },
                            "commits": { "nodes": [{ "commit": { "statusCheckRollup": { "state": "FAILURE" } } }] },
                            "mergeable": "MERGEABLE"
                        }
                    ]
                }
            }
        }"#
        .to_string()
    }

    #[test]
    fn classifies_prs_by_role() {
        let group = parse_response(&sample_response(), "testuser").unwrap();
        assert_eq!(group.mine.len(), 1);
        assert_eq!(group.mine[0].title, "Add feature");
        assert_eq!(group.review_requested.len(), 1);
        assert_eq!(group.review_requested[0].title, "Review this");
        assert_eq!(group.involved.len(), 1);
        assert_eq!(group.involved[0].title, "Involved PR");
    }

    #[test]
    fn parses_check_status() {
        let group = parse_response(&sample_response(), "testuser").unwrap();
        assert_eq!(group.mine[0].check_status, Some(CheckStatus::Success));
        assert_eq!(
            group.review_requested[0].check_status,
            Some(CheckStatus::Pending)
        );
        assert_eq!(group.involved[0].check_status, Some(CheckStatus::Failure));
    }

    #[test]
    fn parses_review_status() {
        let group = parse_response(&sample_response(), "testuser").unwrap();
        assert_eq!(group.mine[0].review_status, Some(ReviewStatus::Approved));
        assert_eq!(group.review_requested[0].review_status, None);
        assert_eq!(
            group.involved[0].review_status,
            Some(ReviewStatus::ChangesRequested)
        );
    }

    #[test]
    fn parses_draft_and_conflicts() {
        let group = parse_response(&sample_response(), "testuser").unwrap();
        assert!(!group.mine[0].is_draft);
        assert!(group.review_requested[0].is_draft);
        assert!(!group.mine[0].has_conflicts);
        assert!(group.review_requested[0].has_conflicts);
    }

    #[test]
    fn handles_missing_check_status() {
        let json = r#"{
            "data": { "search": { "nodes": [{
                "title": "No checks",
                "url": "https://example.com/1",
                "number": 1,
                "isDraft": false,
                "repository": { "nameWithOwner": "org/repo" },
                "author": { "login": "user" },
                "reviewRequests": { "nodes": [] },
                "reviews": { "nodes": [] },
                "commits": { "nodes": [{ "commit": { "statusCheckRollup": null } }] },
                "mergeable": "UNKNOWN"
            }] } }
        }"#;
        let group = parse_response(json, "user").unwrap();
        assert_eq!(group.mine[0].check_status, None);
    }

    #[test]
    fn username_matching_is_case_insensitive() {
        let json = r#"{
            "data": { "search": { "nodes": [{
                "title": "My PR",
                "url": "https://example.com/1",
                "number": 1,
                "isDraft": false,
                "repository": { "nameWithOwner": "org/repo" },
                "author": { "login": "TestUser" },
                "reviewRequests": { "nodes": [] },
                "reviews": { "nodes": [] },
                "commits": { "nodes": [] },
                "mergeable": "MERGEABLE"
            }] } }
        }"#;
        let group = parse_response(json, "testuser").unwrap();
        assert_eq!(group.mine.len(), 1);
    }
}
