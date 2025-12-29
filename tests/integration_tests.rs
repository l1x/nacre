use axum::http::StatusCode;
use axum_test::TestServer;
use nacre::{create_app, AppState};
use std::sync::Arc;
use tempfile::TempDir;

// Helper to create a test server with a temporary beads database
async fn test_server() -> (TestServer, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Initialize a beads database in the temp directory
    let status = std::process::Command::new("bd")
        .current_dir(temp_dir.path())
        .arg("init")
        .status()
        .expect("Failed to run bd init");
    
    assert!(status.success(), "bd init failed");

    let beads_dir = temp_dir.path().join(".beads");
    let mut db_path = None;
    if let Ok(entries) = std::fs::read_dir(&beads_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "db" {
                    db_path = Some(path);
                    break;
                }
            }
        }
    }
    
    let mut state = AppState::new();
    if let Some(path) = db_path {
        state.client = state.client.with_db(path.to_string_lossy().to_string());
    }

    let app = create_app(Arc::new(state));
    let server = TestServer::new(app).unwrap();
    
    (server, temp_dir)
}

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
    let (server, _temp) = test_server().await;
    
    let response = server.get("/").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.text().contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn test_tasks_list() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/tasks").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.text().contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn test_board_view() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/board").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_graph_view() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/graph").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_palette_view() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/palette").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_tasks_list() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/api/issues").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    // Should be valid JSON
    let _json: serde_json::Value = response.json();
}

#[tokio::test]
async fn test_metrics_view() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/metrics").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_prds_list() {
    let (server, _temp) = test_server().await;
    
    let response = server.get("/prds").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}