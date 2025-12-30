use axum::extract::{Path, State};

use crate::templates::*;

pub async fn prds_list(State(state): State<crate::SharedAppState>) -> PrdsListTemplate {
    let mut files_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("docs/prds") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string()
                && name.ends_with(".md")
            {
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files_with_time.push((name, modified));
            }
        }
    }
    // Sort by most recently modified first
    files_with_time.sort_by(|a, b| b.1.cmp(&a.1));
    let files: Vec<String> = files_with_time.into_iter().map(|(name, _)| name).collect();
    PrdsListTemplate {
        project_name: state.project_name.clone(),
        page_title: "PRDs".to_string(),
        active_nav: "prds",
        app_version: state.app_version.clone(),
        files,
    }
}

pub async fn prd_view(
    State(state): State<crate::SharedAppState>,
    Path(filename): Path<String>,
) -> crate::AppResult<PrdViewTemplate> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') || !filename.ends_with(".md") {
        return Err(crate::AppError::BadRequest("Invalid filename".to_string()));
    }

    let path = format!("docs/prds/{}", filename);
    let markdown_input =
        std::fs::read_to_string(&path).map_err(|_| crate::AppError::NotFound(filename.clone()))?;

    let html_output = crate::markdown::render(&markdown_input);

    Ok(PrdViewTemplate {
        project_name: state.project_name.clone(),
        page_title: filename.clone(),
        active_nav: "prds-view",
        app_version: state.app_version.clone(),
        filename,
        content: html_output,
    })
}
