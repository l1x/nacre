//! PRD (Product Requirements Document) integration tests.
//!
//! Tests for PRD listing and detail views

use axum::http::StatusCode;
use crate::common::test_server;

#[tokio::test]
async fn test_prds_list() {
    let (server, _temp) = test_server().await;

    let response = server.get("/prds").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_prd_detail() {
    let (server, _temp) = test_server().await;

    // Test with an existing PRD file
    let response = server.get("/prds/prd-foundation-v1-2025-12-15.md").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.text().contains("<!DOCTYPE html>"));
    assert!(response.text().contains("prd-foundation-v1-2025-12-15.md"));
}

#[tokio::test]
async fn test_prd_detail_not_found() {
    let (server, _temp) = test_server().await;

    let response = server.get("/prds/nonexistent-prd.md").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
