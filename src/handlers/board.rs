use crate::beads;
use crate::templates::*;
use axum::extract::{Query, State};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Default)]
pub struct BoardQuery {
    #[serde(default)]
    pub include_closed: bool,
}

pub async fn board(
    State(state): State<crate::SharedAppState>,
    Query(query): Query<BoardQuery>,
) -> crate::AppResult<BoardTemplate> {
    // Always load all issues for assignee list and dependency resolution
    let every_issue = state.client.list_all_issues()?;

    // The visible issues for board columns
    let all_issues: Vec<beads::Issue> = if query.include_closed {
        every_issue.clone()
    } else {
        every_issue
            .iter()
            .filter(|i| i.status != beads::Status::Closed)
            .cloned()
            .collect()
    };

    // Build a set of closed issue IDs for blocked-status calculation
    let closed_ids: HashSet<&str> = every_issue
        .iter()
        .filter(|i| i.status == beads::Status::Closed)
        .map(|i| i.id.as_str())
        .collect();

    // Build a map of issue ID → status for dependency resolution
    let status_map: HashMap<&str, &beads::Status> = every_issue
        .iter()
        .map(|i| (i.id.as_str(), &i.status))
        .collect();

    // Load all dependencies to determine which issues are blocked
    let all_deps = state.client.list_all_dependencies().unwrap_or_default();

    // An issue is "blocked" if it has a workflow dependency on a non-closed issue
    let mut blocked_ids: HashSet<String> = HashSet::new();
    for dep in &all_deps {
        if dep.dep_type.affects_workflow() {
            let blocker_closed = closed_ids.contains(dep.depends_on_id.as_str())
                || !status_map.contains_key(dep.depends_on_id.as_str());
            if !blocker_closed {
                blocked_ids.insert(dep.issue_id.clone());
            }
        }
    }

    // Collect unique assignees from ALL issues so filter is always available
    let mut assignees: Vec<String> = every_issue
        .iter()
        .filter_map(|i| i.assignee.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    assignees.sort();

    let wrap = |issues: Vec<beads::Issue>| -> Vec<BoardIssue> {
        issues
            .into_iter()
            .map(|issue| {
                let is_blocked = blocked_ids.contains(&issue.id);
                BoardIssue { issue, is_blocked }
            })
            .collect()
    };

    let columns = vec![
        BoardColumn {
            name: "Open".to_string(),
            status: "open".to_string(),
            issues: wrap(
                all_issues
                    .iter()
                    .filter(|i| i.status == beads::Status::Open)
                    .cloned()
                    .collect(),
            ),
        },
        BoardColumn {
            name: "In Progress".to_string(),
            status: "in_progress".to_string(),
            issues: wrap(
                all_issues
                    .iter()
                    .filter(|i| i.status == beads::Status::InProgress)
                    .cloned()
                    .collect(),
            ),
        },
        BoardColumn {
            name: "Blocked".to_string(),
            status: "blocked".to_string(),
            issues: wrap(
                all_issues
                    .iter()
                    .filter(|i| i.status == beads::Status::Blocked)
                    .cloned()
                    .collect(),
            ),
        },
        BoardColumn {
            name: "Deferred".to_string(),
            status: "deferred".to_string(),
            issues: wrap(
                all_issues
                    .iter()
                    .filter(|i| i.status == beads::Status::Deferred)
                    .cloned()
                    .collect(),
            ),
        },
        BoardColumn {
            name: "Closed".to_string(),
            status: "closed".to_string(),
            issues: wrap(
                all_issues
                    .iter()
                    .filter(|i| i.status == beads::Status::Closed)
                    .cloned()
                    .collect(),
            ),
        },
    ];

    Ok(BoardTemplate {
        project_name: state.project_name.clone(),
        page_title: "Board".to_string(),
        active_nav: "board",
        app_version: state.app_version.clone(),
        columns,
        assignees,
        include_closed: query.include_closed,
    })
}
