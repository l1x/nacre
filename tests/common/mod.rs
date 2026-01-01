//! Shared test utilities for integration tests.
//!
//! This module provides common helpers used across all integration test files,
//! enabling parallel development by multiple agents.

use axum_test::TestServer;
use nacre::{create_app, AppState};
use std::sync::Arc;

/// Creates a test server for integration testing.
///
/// Returns a test server that can be used to test endpoints.
/// The server uses a default app state without requiring a beads database.
pub async fn test_server() -> TestServer {
    let state = Arc::new(AppState::new());
    let app = create_app(state);
    TestServer::new(app).unwrap()
}
