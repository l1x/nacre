use axum::extract::{Path, State};

use crate::beads;
use crate::templates::*;

pub async fn epics(State(state): State<crate::AppState>) -> EpicsTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();

    let mut epics: Vec<EpicWithProgress> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic)
        .map(|epic| EpicWithProgress::from_epic(epic, &all_issues, true))
        .collect();

    // Sort epics by most recently updated first
    epics.sort_by(|a, b| b.issue.updated_at.cmp(&a.issue.updated_at));

    EpicsTemplate {
        project_name: state.project_name.clone(),
        page_title: "Epics".to_string(),
        active_nav: "epics",
        epics,
    }
}

pub async fn epic_detail(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
) -> crate::AppResult<EpicDetailTemplate> {
    let all_issues = state.client.list_issues()?;

    // Find the epic
    let epic = all_issues
        .iter()
        .find(|i| i.id == id && i.issue_type == beads::IssueType::Epic)
        .ok_or_else(|| crate::AppError::NotFound(format!("Epic {}", id)))?;

    Ok(EpicDetailTemplate {
        project_name: state.project_name.clone(),
        page_title: id.clone(),
        active_nav: "epics",
        epic: EpicWithProgress::from_epic(epic, &all_issues, true),
    })
}
