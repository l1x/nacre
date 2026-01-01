//! PRD (Product Requirements Document) integration tests.
//!
//! Tests for PRD listing and detail views

use axum::http::StatusCode;
use crate::common::test_server;

#[tokio::test]
async fn test_prds_list() {
    let server = test_server().await;

    let response = server.get("/prds").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_prd_detail_not_found() {
    let server = test_server().await;

    let response = server.get("/prds/nonexistent-prd.md").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
