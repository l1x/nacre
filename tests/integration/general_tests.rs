//! General integration tests.
//!
//! Tests for health check, landing page, and miscellaneous views

use crate::common::test_server;
use axum::http::StatusCode;
use axum_test::TestServer;
use nacre::{AppState, create_app};
use std::sync::Arc;

#[tokio::test]
async fn test_health_check() {
    let state = Arc::new(AppState::new());
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(response.text(), "OK");
}

#[tokio::test]
async fn test_landing_page() {
    let server = test_server().await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.text().contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn test_graph_view() {
    let server = test_server().await;

    let response = server.get("/graph").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_palette_view() {
    let server = test_server().await;

    let response = server.get("/palette").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
