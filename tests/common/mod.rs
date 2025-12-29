//! Shared test utilities for integration tests.
//!
//! This module provides common helpers used across all integration test files,
//! enabling parallel development by multiple agents.

use axum_test::TestServer;
use nacre::{create_app, AppState};
use std::sync::Arc;
use tempfile::TempDir;

/// Creates a test server with a temporary beads database.
///
/// Returns both the server and the temp directory (which must be kept alive
/// for the duration of the test to prevent cleanup).
pub async fn test_server() -> (TestServer, TempDir) {
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

/// Creates a test issue using the bd CLI.
///
/// # Arguments
/// * `temp_dir` - The temporary directory containing the beads database
/// * `title` - The issue title
/// * `issue_type` - Optional issue type (e.g., "task", "bug", "feature")
/// * `priority` - Optional priority level (0-4)
///
/// # Returns
/// The created issue ID as a string
pub fn create_test_issue(
    temp_dir: &TempDir,
    title: &str,
    issue_type: Option<&str>,
    priority: Option<u8>,
) -> String {
    let mut cmd = std::process::Command::new("bd");
    cmd.current_dir(temp_dir.path())
        .arg("create")
        .arg("--title")
        .arg(title)
        .arg("--silent");

    if let Some(t) = issue_type {
        cmd.arg("--type").arg(t);
    }
    if let Some(p) = priority {
        cmd.arg("--priority").arg(p.to_string());
    }

    let output = cmd.output().expect("Failed to create test issue");
    assert!(
        output.status.success(),
        "bd create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
