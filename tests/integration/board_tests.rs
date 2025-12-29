//! Board view integration tests.
//!
//! Tests for the Kanban board view

use axum::http::StatusCode;
use crate::common::test_server;

#[tokio::test]
async fn test_board_view() {
    let (server, _temp) = test_server().await;

    let response = server.get("/board").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
