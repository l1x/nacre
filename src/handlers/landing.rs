use axum::extract::State;
use time::OffsetDateTime;

use crate::beads::{Issue, IssueType, Status};
use crate::handlers::metrics::{build_tickets_chart, generate_date_range};
use crate::templates::*;

// ============================================================================
// Pure Functions - testable without mocking
// ============================================================================

/// Calculate project statistics from issues
pub fn calculate_project_stats(issues: &[Issue]) -> ProjectStats {
    ProjectStats {
        total: issues.len(),
        open: issues.iter().filter(|i| i.status == Status::Open).count(),
        in_progress: issues
            .iter()
            .filter(|i| i.status == Status::InProgress)
            .count(),
        blocked: issues
            .iter()
            .filter(|i| i.status == Status::Blocked)
            .count(),
        closed: issues.iter().filter(|i| i.status == Status::Closed).count(),
    }
}

/// Get open epics with their progress, sorted by percent complete (least first)
pub fn build_epic_progress_list(issues: &[Issue]) -> Vec<EpicWithProgress> {
    let mut epics: Vec<EpicWithProgress> = issues
        .iter()
        .filter(|i| i.issue_type == IssueType::Epic && i.status != Status::Closed)
        .map(|epic| EpicWithProgress::from_epic(epic, issues, false))
        .collect();

    epics.sort_by(|a, b| {
        a.percent
            .partial_cmp(&b.percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    epics
}

/// Get issues by status with optional limit
pub fn get_issues_by_status(issues: &[Issue], status: Status, limit: usize) -> Vec<Issue> {
    issues
        .iter()
        .filter(|i| i.status == status)
        .take(limit)
        .cloned()
        .collect()
}

// ============================================================================
// Handler - thin orchestration layer
// ============================================================================

pub async fn landing(
    State(state): State<crate::SharedAppState>,
) -> crate::AppResult<LandingTemplate> {
    let all_issues = state.client.list_issues()?;

    // Use pure functions for all calculations
    let now = OffsetDateTime::now_utc();
    let dates = generate_date_range(now);

    let stats = calculate_project_stats(&all_issues);
    let epics = build_epic_progress_list(&all_issues);
    let blocked = get_issues_by_status(&all_issues, Status::Blocked, 5);
    let in_progress = get_issues_by_status(&all_issues, Status::InProgress, 5);
    let tickets_chart = build_tickets_chart(&all_issues, &dates);

    Ok(LandingTemplate {
        project_name: state.project_name.clone(),
        page_title: String::new(),
        active_nav: "dashboard",
        app_version: state.app_version.clone(),
        stats,
        epics,
        blocked,
        in_progress,
        tickets_chart,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_issue(id: &str, status: Status, issue_type: IssueType) -> Issue {
        Issue {
            id: id.to_string(),
            title: format!("Test {}", id),
            status,
            priority: Some(2),
            issue_type,
            created_at: time::OffsetDateTime::now_utc(),
            updated_at: time::OffsetDateTime::now_utc(),
            closed_at: None,
            assignee: None,
            labels: None,
            description: None,
            acceptance_criteria: None,
            close_reason: None,
            estimate: None,
            dependencies: vec![],
        }
    }

    #[test]
    fn test_calculate_project_stats() {
        let issues = vec![
            make_test_issue("test-1", Status::Open, IssueType::Task),
            make_test_issue("test-2", Status::InProgress, IssueType::Task),
            make_test_issue("test-3", Status::Blocked, IssueType::Task),
            make_test_issue("test-4", Status::Closed, IssueType::Task),
            make_test_issue("test-5", Status::Closed, IssueType::Task),
        ];

        let stats = calculate_project_stats(&issues);

        assert_eq!(stats.total, 5);
        assert_eq!(stats.open, 1);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.blocked, 1);
        assert_eq!(stats.closed, 2);
    }

    #[test]
    fn test_calculate_project_stats_empty() {
        let stats = calculate_project_stats(&[]);

        assert_eq!(stats.total, 0);
        assert_eq!(stats.open, 0);
        assert_eq!(stats.in_progress, 0);
        assert_eq!(stats.blocked, 0);
        assert_eq!(stats.closed, 0);
    }

    #[test]
    fn test_get_issues_by_status() {
        let issues = vec![
            make_test_issue("test-1", Status::Blocked, IssueType::Task),
            make_test_issue("test-2", Status::Blocked, IssueType::Task),
            make_test_issue("test-3", Status::Blocked, IssueType::Task),
            make_test_issue("test-4", Status::Open, IssueType::Task),
        ];

        let blocked = get_issues_by_status(&issues, Status::Blocked, 2);

        assert_eq!(blocked.len(), 2);
        assert_eq!(blocked[0].id, "test-1");
        assert_eq!(blocked[1].id, "test-2");
    }

    #[test]
    fn test_build_epic_progress_list() {
        let issues = vec![
            make_test_issue("epic-1", Status::Open, IssueType::Epic),
            make_test_issue("epic-2", Status::InProgress, IssueType::Epic),
            make_test_issue("epic-3", Status::Closed, IssueType::Epic), // Should be excluded
            make_test_issue("task-1", Status::Open, IssueType::Task),
        ];

        let epics = build_epic_progress_list(&issues);

        assert_eq!(epics.len(), 2);
        // Both have 0% progress (no children), so order may vary
        assert!(epics.iter().all(|e| e.issue.issue_type == IssueType::Epic));
        assert!(epics.iter().all(|e| e.issue.status != Status::Closed));
    }
}
