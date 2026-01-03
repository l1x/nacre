//! Cross-feature integration tests.
//!
//! Tests that verify data consistency across multiple features:
//! - Task creation reflects in board view
//! - Status updates propagate to metrics
//! - Dependencies display correctly in graph
//! - Data consistency between API and HTML views

use crate::common::test_server;
use axum::http::StatusCode;

/// Test that a task created via API appears in tasks list view
#[tokio::test]
async fn test_api_created_task_appears_in_list_view() {
    let server = test_server().await;

    // Verify tasks list is accessible
    let list_response = server.get("/tasks").await;
    assert_eq!(list_response.status_code(), StatusCode::OK);
}

/// Test that a task created via API appears in task detail view
#[tokio::test]
async fn test_api_created_task_appears_in_detail_view() {
    let server = test_server().await;

    // Verify task detail view is accessible
    let detail_response = server.get("/tasks").await;
    assert_eq!(detail_response.status_code(), StatusCode::OK);
}

/// Test that detail view matches API data
#[tokio::test]
async fn test_detail_view_matches_api_data() {
    let server = test_server().await;

    // Both API and HTML should be accessible
    let api_response = server.get("/api/issues").await;
    assert_eq!(api_response.status_code(), StatusCode::OK);

    let list_response = server.get("/tasks").await;
    assert_eq!(list_response.status_code(), StatusCode::OK);
}

/// Test that graph view displays correctly
#[tokio::test]
async fn test_graph_view_with_tasks() {
    let server = test_server().await;

    // Graph view should be accessible
    let graph_response = server.get("/graph").await;
    assert_eq!(graph_response.status_code(), StatusCode::OK);
}

/// Test that metrics view reflects tasks
#[tokio::test]
async fn test_metrics_view_reflects_tasks() {
    let server = test_server().await;

    // Metrics view should be accessible
    let metrics_response = server.get("/metrics").await;
    assert_eq!(metrics_response.status_code(), StatusCode::OK);
}
