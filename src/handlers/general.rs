use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use include_dir::{Dir, include_dir};

use crate::templates::*;

// Embed entire frontend/public directory at compile time
static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/public");

// Cache for 1 year (immutable content versioned by ETag)
const CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

fn content_type(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("svg") => "image/svg+xml",
        Some("html") => "text/html",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn make_etag(filename: &str) -> String {
    format!("\"{}-{}\"", env!("CARGO_PKG_VERSION"), filename)
}

/// Serve a static file from the embedded ASSETS directory
fn serve_asset(filename: &str, headers: &HeaderMap) -> Response {
    let Some(file) = ASSETS.get_file(filename) else {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    };

    let etag = make_etag(filename);

    // Check If-None-Match for caching
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
        && if_none_match.as_bytes() == etag.as_bytes()
    {
        return (StatusCode::NOT_MODIFIED, HeaderMap::new(), "").into_response();
    }

    let content = file.contents_utf8().unwrap_or("");
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type(filename)));
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL));
    if let Ok(etag_value) = HeaderValue::from_str(&etag) {
        response_headers.insert(header::ETAG, etag_value);
    }

    (response_headers, content).into_response()
}

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn serve_css(headers: HeaderMap) -> Response {
    serve_asset("style.css", &headers)
}

pub async fn serve_autumnus_dark(headers: HeaderMap) -> Response {
    serve_asset("autumnus.dark.css", &headers)
}

pub async fn serve_autumnus_light(headers: HeaderMap) -> Response {
    serve_asset("autumnus.light.css", &headers)
}

pub async fn serve_js(headers: HeaderMap) -> Response {
    serve_asset("app.js", &headers)
}

pub async fn serve_favicon(headers: HeaderMap) -> Response {
    serve_asset("favicon.svg", &headers)
}

/// Generic static file handler for future use with wildcard routes
#[allow(dead_code)]
pub async fn serve_static(Path(filename): Path<String>, headers: HeaderMap) -> Response {
    serve_asset(&filename, &headers)
}

pub async fn graph(
    State(state): State<crate::SharedAppState>,
) -> crate::AppResult<GraphTemplate> {
    let all_issues = state.client.list_issues()?;

    // Get all epics for the selector
    let epics: Vec<EpicSummary> = all_issues
        .iter()
        .filter(|i| i.issue_type == crate::beads::IssueType::Epic && i.status != crate::beads::Status::Tombstone)
        .map(|i| EpicSummary {
            id: i.id.clone(),
            title: i.title.clone(),
            selected: false,
        })
        .collect();

    Ok(GraphTemplate {
        project_name: state.project_name.clone(),
        page_title: "Graph".to_string(),
        active_nav: "graph",
        app_version: state.app_version.clone(),
        epics,
        tree: vec![],
    })
}

pub async fn graph_epic(
    State(state): State<crate::SharedAppState>,
    Path(epic_id): Path<String>,
) -> crate::AppResult<GraphTemplate> {
    let all_issues = state.client.list_issues()?;

    // Get all epics for the selector
    let epics: Vec<EpicSummary> = all_issues
        .iter()
        .filter(|i| i.issue_type == crate::beads::IssueType::Epic && i.status != crate::beads::Status::Tombstone)
        .map(|i| EpicSummary {
            id: i.id.clone(),
            title: i.title.clone(),
            selected: i.id == epic_id,
        })
        .collect();

    // Filter to just this epic and its descendants
    let prefix = format!("{}.", epic_id);
    let descendants: Vec<crate::beads::Issue> = all_issues
        .into_iter()
        .filter(|i| {
            i.status != crate::beads::Status::Tombstone
                && (i.id == epic_id || i.id.starts_with(&prefix))
        })
        .collect();

    // Build tree
    let tree = super::tasks::build_issue_tree(&descendants);

    Ok(GraphTemplate {
        project_name: state.project_name.clone(),
        page_title: "Graph".to_string(),
        active_nav: "graph",
        app_version: state.app_version.clone(),
        epics,
        tree,
    })
}

pub async fn palette(State(state): State<crate::SharedAppState>) -> PaletteTemplate {
    PaletteTemplate {
        project_name: state.project_name.clone(),
        page_title: "Design System".to_string(),
        active_nav: "palette",
        app_version: state.app_version.clone(),
    }
}
