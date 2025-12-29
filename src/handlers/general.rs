use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::templates::*;

// Embed static assets at compile time
const STYLE_CSS: &str = include_str!("../../frontend/public/style.css");
const APP_JS: &str = include_str!("../../frontend/public/app.js");
const FAVICON_SVG: &str = include_str!("../../frontend/public/favicon.svg");

// Generate ETags at compile time based on version
const CSS_ETAG: &str = concat!("\"", env!("CARGO_PKG_VERSION"), "-css\"");
const JS_ETAG: &str = concat!("\"", env!("CARGO_PKG_VERSION"), "-js\"");
const FAVICON_ETAG: &str = concat!("\"", env!("CARGO_PKG_VERSION"), "-favicon\"");

// Cache for 1 year (immutable content versioned by ETag)
const CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn serve_css(headers: HeaderMap) -> impl IntoResponse {
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.as_bytes() == CSS_ETAG.as_bytes() {
            return (StatusCode::NOT_MODIFIED, HeaderMap::new(), "").into_response();
        }
    }
    (
        [
            (header::CONTENT_TYPE, "text/css"),
            (header::CACHE_CONTROL, CACHE_CONTROL),
            (header::ETAG, CSS_ETAG),
        ],
        STYLE_CSS,
    )
        .into_response()
}

pub async fn serve_js(headers: HeaderMap) -> impl IntoResponse {
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.as_bytes() == JS_ETAG.as_bytes() {
            return (StatusCode::NOT_MODIFIED, HeaderMap::new(), "").into_response();
        }
    }
    (
        [
            (header::CONTENT_TYPE, "application/javascript"),
            (header::CACHE_CONTROL, CACHE_CONTROL),
            (header::ETAG, JS_ETAG),
        ],
        APP_JS,
    )
        .into_response()
}

pub async fn serve_favicon(headers: HeaderMap) -> impl IntoResponse {
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.as_bytes() == FAVICON_ETAG.as_bytes() {
            return (StatusCode::NOT_MODIFIED, HeaderMap::new(), "").into_response();
        }
    }
    (
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, CACHE_CONTROL),
            (header::ETAG, FAVICON_ETAG),
        ],
        FAVICON_SVG,
    )
        .into_response()
}

pub async fn graph(State(state): State<crate::SharedAppState>) -> GraphTemplate {
    GraphTemplate {
        project_name: state.project_name.clone(),
        page_title: "Graph".to_string(),
        active_nav: "graph",
        app_version: state.app_version.clone(),
    }
}

pub async fn palette(State(state): State<crate::SharedAppState>) -> PaletteTemplate {
    PaletteTemplate {
        project_name: state.project_name.clone(),
        page_title: "Design System".to_string(),
        active_nav: "palette",
        app_version: state.app_version.clone(),
    }
}
