//! Board view integration tests.
//!
//! Tests for the Kanban board view

use crate::common::test_server;
use axum::http::StatusCode;

#[tokio::test]
async fn test_board_view() {
    let server = test_server().await;

    let response = server.get("/board").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
