use axum::http::StatusCode;
use axum_test::TestServer;
use nacre::{AppState, create_app};
use std::sync::Arc;

#[tokio::test]
async fn test_path_traversal_protection() {
    let state = Arc::new(AppState::new());
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Try to access Cargo.toml via path traversal
    // The filename parameter in the route is /prds/:filename
    // We expect the server to reject ".."
    let _response = server.get("/prds/../../Cargo.toml").await;

    // Axum/Hyper might normalize ".." before it reaches the handler if not careful,
    // but typically it doesn't cross route segments if defined as :filename.
    // However, :filename matches a single segment.
    // If we request /prds/../../Cargo.toml, the router might not even match /prds/:filename
    // because that expects exactly two segments: /prds/something.

    // If we encode it: /prds/%2e%2e%2f%2e%2e%2fCargo.toml
    // The decoded filename would be "../../Cargo.toml".

    // Let's try encoded traversal
    let response = server.get("/prds/%2e%2e%2f%2e%2e%2fCargo.toml").await;

    // It should be 400 Bad Request or 404 Not Found, but definitely NOT 200
    assert_ne!(response.status_code(), StatusCode::OK);
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_valid_prd_access() {
    let state = Arc::new(AppState::new());
    let app = create_app(state);
    let server = TestServer::new(app).unwrap();

    // Try to access a known existing PRD
    // We need to pick one that exists in the repo.
    // From file listing: docs/prds/prd-nacre-v1-2025-12-15.md

    let response = server.get("/prds/prd-nacre-v1-2025-12-15.md").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.text().contains("PRD: Nacre"));
}
