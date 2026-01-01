//! Static assets integration tests.
//!
//! Tests for CSS, JavaScript, and favicon serving with caching

use axum::http::StatusCode;
use crate::common::test_server;

#[tokio::test]
async fn test_static_assets() {
    let server = test_server().await;

    // Test CSS endpoint with ETag
    let css_response = server.get("/style.css").await;
    assert_eq!(css_response.status_code(), StatusCode::OK);
    let content_type = css_response.headers().get("content-type").unwrap();
    assert_eq!(content_type.to_str().unwrap(), "text/css");
    assert!(css_response.text().contains("font-family"));

    // Test with If-None-Match header
    let etag = css_response.headers().get("etag").unwrap();
    let cached_response = server
        .get("/style.css")
        .add_header("if-none-match", etag.to_str().unwrap())
        .await;
    assert_eq!(cached_response.status_code(), StatusCode::NOT_MODIFIED);

    // Test JS endpoint
    let js_response = server.get("/app.js").await;
    assert_eq!(js_response.status_code(), StatusCode::OK);
    let js_content_type = js_response.headers().get("content-type").unwrap();
    assert_eq!(js_content_type.to_str().unwrap(), "application/javascript");

    // Test favicon endpoint
    let favicon_response = server.get("/favicon.svg").await;
    assert_eq!(favicon_response.status_code(), StatusCode::OK);
    let favicon_content_type = favicon_response.headers().get("content-type").unwrap();
    assert_eq!(favicon_content_type.to_str().unwrap(), "image/svg+xml");
}

#[tokio::test]
async fn test_css_caching() {
    let server = test_server().await;

    // First request should return full content
    let first_response = server.get("/style.css").await;
    assert_eq!(first_response.status_code(), StatusCode::OK);

    let etag = first_response.headers().get("etag");
    assert!(etag.is_some(), "CSS response should include ETag header");

    // Second request with matching ETag should return 304
    let cached_response = server
        .get("/style.css")
        .add_header("if-none-match", etag.unwrap().to_str().unwrap())
        .await;
    assert_eq!(cached_response.status_code(), StatusCode::NOT_MODIFIED);
}

#[tokio::test]
async fn test_js_content_type() {
    let server = test_server().await;

    let response = server.get("/app.js").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type.to_str().unwrap(), "application/javascript");
}

#[tokio::test]
async fn test_favicon_svg() {
    let server = test_server().await;

    let response = server.get("/favicon.svg").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type.to_str().unwrap(), "image/svg+xml");
}
