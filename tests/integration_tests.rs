//! Integration tests for Nacre web application.
//!
//! This file serves as the entry point for all integration tests.
//! Tests are organized into separate modules by feature area to enable
//! parallel development by multiple agents without merge conflicts.
//!
//! ## Module Structure
//!
//! - `common/` - Shared test utilities (test_server, create_test_issue)
//! - `integration/` - Feature-specific test modules:
//!   - `api_tests` - REST API endpoints
//!   - `task_views_tests` - Task list, detail, edit views
//!   - `general_tests` - Health check, landing, graph, palette
//!   - `board_tests` - Kanban board view
//!   - `metrics_tests` - Metrics dashboard
//!   - `prd_tests` - PRD listing and detail views
//!   - `static_assets_tests` - CSS, JS, favicon with caching

mod common;
mod integration;
