use axum::{
    extract::State,
    http::header,
    response::IntoResponse,
};

use crate::templates::*;

// Embed static assets at compile time
const STYLE_CSS: &str = include_str!("../../frontend/public/style.css");
const APP_JS: &str = include_str!("../../frontend/public/app.js");
const FAVICON_SVG: &str = include_str!("../../frontend/public/favicon.svg");

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn serve_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

pub async fn serve_js() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/javascript")], APP_JS)
}

pub async fn serve_favicon() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}

pub async fn graph(State(state): State<crate::AppState>) -> GraphTemplate {
    GraphTemplate {
        project_name: state.project_name.clone(),
        page_title: "Graph".to_string(),
        active_nav: "graph",
    }
}
