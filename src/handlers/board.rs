use crate::beads;
use crate::templates::*;
use axum::extract::State;

pub async fn board(State(state): State<crate::SharedAppState>) -> crate::AppResult<BoardTemplate> {
    let all_issues = state.client.list_issues()?;

    let columns = vec![
        BoardColumn {
            name: "Open".to_string(),
            status: "open".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Open)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "In Progress".to_string(),
            status: "in_progress".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::InProgress)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Blocked".to_string(),
            status: "blocked".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Blocked)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Deferred".to_string(),
            status: "deferred".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Deferred)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Closed".to_string(),
            status: "closed".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Closed)
                .cloned()
                .collect(),
        },
    ];

    Ok(BoardTemplate {
        project_name: state.project_name.clone(),
        page_title: "Board".to_string(),
        active_nav: "board",
        app_version: state.app_version.clone(),
        columns,
    })
}
