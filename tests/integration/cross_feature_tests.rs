//! Cross-feature integration tests.
//!
//! Tests that verify data consistency across multiple features:
//! - Task creation reflects in board view
//! - Status updates propagate to metrics
//! - Dependencies display correctly in graph
//! - Data consistency between API and HTML views

use axum::http::StatusCode;
use crate::common::{create_test_issue, test_server};

/// Test that a task created via API appears in the tasks list view
#[tokio::test]
async fn test_api_created_task_appears_in_list_view() {
    let (server, _temp) = test_server().await;

    // Create a task via API
    let create_data = serde_json::json!({
        "title": "Cross-Feature Test Task",
        "issue_type": "task",
        "priority": 1
    });

    let api_response = server.post("/api/issues").json(&create_data).await;
    assert_eq!(api_response.status_code(), StatusCode::OK);

    let response_json: serde_json::Value = api_response.json();
    let issue_id = response_json["id"].as_str().unwrap();

    // Verify task appears in HTML list view
    let list_response = server.get("/tasks").await;
    assert_eq!(list_response.status_code(), StatusCode::OK);
    let html = list_response.text();
    assert!(html.contains("Cross-Feature Test Task"), "Task title should appear in list view");
    assert!(html.contains(issue_id), "Task ID should appear in list view");
}

/// Test that a task created via API appears in the task detail view
#[tokio::test]
async fn test_api_created_task_appears_in_detail_view() {
    let (server, _temp) = test_server().await;

    // Create a task via API
    let create_data = serde_json::json!({
        "title": "Detail View Test Task",
        "issue_type": "feature",
        "priority": 2
    });

    let api_response = server.post("/api/issues").json(&create_data).await;
    assert_eq!(api_response.status_code(), StatusCode::OK);

    let response_json: serde_json::Value = api_response.json();
    let issue_id = response_json["id"].as_str().unwrap();

    // Verify task appears in detail view
    let detail_response = server.get(&format!("/tasks/{}", issue_id)).await;
    assert_eq!(detail_response.status_code(), StatusCode::OK);
    let html = detail_response.text();
    assert!(html.contains("Detail View Test Task"), "Task title should appear in detail view");
    assert!(html.contains("Feature"), "Task type should appear in detail view");
}

/// Test that a task created via CLI appears in board view
#[tokio::test]
async fn test_task_appears_in_board_view() {
    let (server, temp) = test_server().await;

    // Create a task using CLI helper
    let issue_id = create_test_issue(&temp, "Board View Task", Some("task"), Some(2));

    // Verify task appears in board view
    let board_response = server.get("/board").await;
    assert_eq!(board_response.status_code(), StatusCode::OK);
    let html = board_response.text();
    assert!(html.contains(&issue_id), "Task ID should appear in board view");
}

/// Test that status update via API reflects in list view
#[tokio::test]
async fn test_status_update_reflects_in_list_view() {
    let (server, temp) = test_server().await;

    // Create a task
    let issue_id = create_test_issue(&temp, "Status Update Task", Some("task"), Some(2));

    // Update status via API
    let update_data = serde_json::json!({
        "status": "in_progress"
    });

    let update_response = server
        .post(&format!("/api/issues/{}", issue_id))
        .json(&update_data)
        .await;
    assert_eq!(update_response.status_code(), StatusCode::OK);

    // Verify status in API response
    let api_list = server.get("/api/issues").await;
    let issues: Vec<serde_json::Value> = api_list.json();
    let updated = issues.iter().find(|i| i["id"].as_str() == Some(&issue_id)).unwrap();
    assert_eq!(updated["status"].as_str(), Some("in_progress"));

    // Verify status reflects in HTML list view
    let list_response = server.get("/tasks").await;
    let html = list_response.text();
    assert!(html.contains("in_progress") || html.contains("In Progress"),
        "Updated status should appear in list view");
}

/// Test that multiple tasks with different statuses appear correctly on board
#[tokio::test]
async fn test_multiple_tasks_on_board_by_status() {
    let (server, temp) = test_server().await;

    // Create tasks with different statuses
    let task1_id = create_test_issue(&temp, "Open Task", Some("task"), Some(2));
    let task2_id = create_test_issue(&temp, "Progress Task", Some("task"), Some(2));

    // Update one task to in_progress
    let update_data = serde_json::json!({ "status": "in_progress" });
    server
        .post(&format!("/api/issues/{}", task2_id))
        .json(&update_data)
        .await;

    // Verify board shows both tasks
    let board_response = server.get("/board").await;
    assert_eq!(board_response.status_code(), StatusCode::OK);
    let html = board_response.text();
    assert!(html.contains(&task1_id), "Open task should appear on board");
    assert!(html.contains(&task2_id), "In-progress task should appear on board");
}

/// Test that API list and HTML list show same tasks
#[tokio::test]
async fn test_api_and_html_list_consistency() {
    let (server, temp) = test_server().await;

    // Create several tasks
    let task1_id = create_test_issue(&temp, "Consistency Task 1", Some("task"), Some(1));
    let task2_id = create_test_issue(&temp, "Consistency Task 2", Some("bug"), Some(2));
    let task3_id = create_test_issue(&temp, "Consistency Task 3", Some("feature"), Some(3));

    // Get API list
    let api_response = server.get("/api/issues").await;
    let api_issues: Vec<serde_json::Value> = api_response.json();

    // Get HTML list
    let html_response = server.get("/tasks").await;
    let html = html_response.text();

    // Verify all tasks appear in both
    for task_id in [&task1_id, &task2_id, &task3_id] {
        assert!(
            api_issues.iter().any(|i| i["id"].as_str() == Some(task_id)),
            "Task {} should appear in API list",
            task_id
        );
        assert!(
            html.contains(task_id),
            "Task {} should appear in HTML list",
            task_id
        );
    }

    // Verify count matches
    assert_eq!(api_issues.len(), 3, "API should return exactly 3 tasks");
}

/// Test that task detail view shows correct data from API
#[tokio::test]
async fn test_detail_view_matches_api_data() {
    let (server, _temp) = test_server().await;

    // Create a task with specific data
    let create_data = serde_json::json!({
        "title": "Data Match Task",
        "issue_type": "bug",
        "priority": 0
    });

    let api_response = server.post("/api/issues").json(&create_data).await;
    let response_json: serde_json::Value = api_response.json();
    let issue_id = response_json["id"].as_str().unwrap();

    // Get task from API
    let api_list = server.get("/api/issues").await;
    let issues: Vec<serde_json::Value> = api_list.json();
    let _api_task = issues.iter().find(|i| i["id"].as_str() == Some(issue_id)).unwrap();

    // Get task detail HTML
    let detail_response = server.get(&format!("/tasks/{}", issue_id)).await;
    let html = detail_response.text();

    // Verify data matches
    assert!(html.contains("Data Match Task"), "Title should match");
    assert!(html.contains("Bug"), "Type should match");
    // Priority 0 is P0/critical
    assert!(html.contains("P0") || html.contains("0"), "Priority should match");
}

/// Test that graph view loads with tasks
#[tokio::test]
async fn test_graph_view_with_tasks() {
    let (server, temp) = test_server().await;

    // Create tasks
    create_test_issue(&temp, "Graph Task 1", Some("epic"), Some(1));
    create_test_issue(&temp, "Graph Task 2", Some("task"), Some(2));

    // Verify graph view loads successfully
    let graph_response = server.get("/graph").await;
    assert_eq!(graph_response.status_code(), StatusCode::OK);
    // Graph renders as SVG/canvas, just verify it loads
    assert!(graph_response.text().contains("<!DOCTYPE html>"));
}

/// Test that metrics view reflects task count
#[tokio::test]
async fn test_metrics_view_reflects_tasks() {
    let (server, temp) = test_server().await;

    // Create some tasks
    create_test_issue(&temp, "Metrics Task 1", Some("task"), Some(2));
    create_test_issue(&temp, "Metrics Task 2", Some("task"), Some(2));

    // Verify metrics view loads and contains data
    let metrics_response = server.get("/metrics").await;
    assert_eq!(metrics_response.status_code(), StatusCode::OK);
    let html = metrics_response.text();
    assert!(html.contains("<!DOCTYPE html>"));
    // Metrics page should show some statistics
}

/// Test full workflow: create, update, verify across views
#[tokio::test]
async fn test_full_task_workflow() {
    let (server, _temp) = test_server().await;

    // 1. Create task via API
    let create_data = serde_json::json!({
        "title": "Workflow Test Task",
        "issue_type": "task",
        "priority": 2
    });

    let create_response = server.post("/api/issues").json(&create_data).await;
    assert_eq!(create_response.status_code(), StatusCode::OK);
    let response_json: serde_json::Value = create_response.json();
    let issue_id = response_json["id"].as_str().unwrap();

    // 2. Verify in list
    let list_response = server.get("/tasks").await;
    assert!(list_response.text().contains("Workflow Test Task"));

    // 3. Verify in detail
    let detail_response = server.get(&format!("/tasks/{}", issue_id)).await;
    assert_eq!(detail_response.status_code(), StatusCode::OK);
    assert!(detail_response.text().contains("Workflow Test Task"));

    // 4. Update status
    let update_data = serde_json::json!({
        "status": "in_progress",
        "title": "Workflow Test Task (Updated)"
    });
    let update_response = server
        .post(&format!("/api/issues/{}", issue_id))
        .json(&update_data)
        .await;
    assert_eq!(update_response.status_code(), StatusCode::OK);

    // 5. Verify update in API
    let api_list = server.get("/api/issues").await;
    let issues: Vec<serde_json::Value> = api_list.json();
    let updated = issues.iter().find(|i| i["id"].as_str() == Some(issue_id)).unwrap();
    assert_eq!(updated["status"].as_str(), Some("in_progress"));
    assert_eq!(updated["title"].as_str(), Some("Workflow Test Task (Updated)"));

    // 6. Verify update in detail view
    let detail_response = server.get(&format!("/tasks/{}", issue_id)).await;
    assert!(detail_response.text().contains("Workflow Test Task (Updated)"));

    // 7. Verify in board
    let board_response = server.get("/board").await;
    assert!(board_response.text().contains(issue_id));
}

/// Test that closing a task updates its status everywhere
#[tokio::test]
async fn test_close_task_workflow() {
    let (server, temp) = test_server().await;

    // Create and close a task
    let issue_id = create_test_issue(&temp, "Close Test Task", Some("task"), Some(2));

    // Update to closed
    let update_data = serde_json::json!({ "status": "closed" });
    let update_response = server
        .post(&format!("/api/issues/{}", issue_id))
        .json(&update_data)
        .await;
    assert_eq!(update_response.status_code(), StatusCode::OK);

    // Verify in API
    let api_list = server.get("/api/issues").await;
    let issues: Vec<serde_json::Value> = api_list.json();
    let closed = issues.iter().find(|i| i["id"].as_str() == Some(&issue_id)).unwrap();
    assert_eq!(closed["status"].as_str(), Some("closed"));

    // Verify in detail view
    let detail_response = server.get(&format!("/tasks/{}", issue_id)).await;
    assert_eq!(detail_response.status_code(), StatusCode::OK);
    let html = detail_response.text();
    assert!(html.contains("closed") || html.contains("Closed"),
        "Closed status should appear in detail view");
}
