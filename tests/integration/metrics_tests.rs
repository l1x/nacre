//! Metrics view integration tests.
//!
//! Tests for the metrics dashboard

use axum::http::StatusCode;
use crate::common::test_server;

#[tokio::test]
async fn test_metrics_view() {
    let server = test_server().await;

    let response = server.get("/metrics").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
