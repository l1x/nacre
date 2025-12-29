use axum::extract::State;

use crate::beads;
use crate::templates::*;

pub async fn landing(State(state): State<crate::SharedAppState>) -> crate::AppResult<LandingTemplate> {
    let all_issues = state.client.list_issues()?;

    // Calculate stats
    let stats = ProjectStats {
        total: all_issues.len(),
        open: all_issues
            .iter()
            .filter(|i| i.status == beads::Status::Open)
            .count(),
        in_progress: all_issues
            .iter()
            .filter(|i| i.status == beads::Status::InProgress)
            .count(),
        blocked: all_issues
            .iter()
            .filter(|i| i.status == beads::Status::Blocked)
            .count(),
        closed: all_issues
            .iter()
            .filter(|i| i.status == beads::Status::Closed)
            .count(),
    };

    // Get epics with progress
    let mut epics: Vec<EpicWithProgress> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic && i.status != beads::Status::Closed)
        .map(|epic| EpicWithProgress::from_epic(epic, &all_issues, false))
        .collect();

    // Sort epics by percent complete (least complete first to highlight work needed)
    epics.sort_by(|a, b| {
        a.percent
            .partial_cmp(&b.percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Get blocked issues (limit to 5)
    let blocked: Vec<beads::Issue> = all_issues
        .iter()
        .filter(|i| i.status == beads::Status::Blocked)
        .take(5)
        .cloned()
        .collect();

    // Get in progress issues (limit to 5)
    let in_progress: Vec<beads::Issue> = all_issues
        .iter()
        .filter(|i| i.status == beads::Status::InProgress)
        .take(5)
        .cloned()
        .collect();

    // Build chart data for the last 7 days
    let now_dt = chrono::Utc::now();
    let start_dt = now_dt - chrono::Duration::days(6); // 7 days including today

    // Collect all dates
    let mut dates: Vec<chrono::NaiveDate> = Vec::new();
    let mut curr = start_dt.date_naive();
    while curr <= now_dt.date_naive() {
        dates.push(curr);
        if let Some(next) = curr.succ_opt() {
            curr = next;
        } else {
            break;
        }
    }
    let labels: Vec<String> = dates.iter().map(|d| d.format("%m.%d").to_string()).collect();

    // --- Tickets Activity Chart ---
    let mut created_by_day: std::collections::HashMap<chrono::NaiveDate, usize> = std::collections::HashMap::new();
    let mut resolved_by_day: std::collections::HashMap<chrono::NaiveDate, usize> = std::collections::HashMap::new();
    for d in &dates {
        created_by_day.insert(*d, 0);
        resolved_by_day.insert(*d, 0);
    }
    for issue in &all_issues {
        let created_date = issue.created_at.date_naive();
        if created_date >= start_dt.date_naive() && created_date <= now_dt.date_naive() {
            *created_by_day.entry(created_date).or_insert(0) += 1;
        }
        if let Some(closed_at) = issue.closed_at {
            let resolved_date = closed_at.date_naive();
            if resolved_date >= start_dt.date_naive() && resolved_date <= now_dt.date_naive() {
                *resolved_by_day.entry(resolved_date).or_insert(0) += 1;
            }
        }
    }
    let created_values: Vec<f64> = dates.iter().map(|d| *created_by_day.get(d).unwrap_or(&0) as f64).collect();
    let resolved_values: Vec<f64> = dates.iter().map(|d| *resolved_by_day.get(d).unwrap_or(&0) as f64).collect();
    let tickets_max = created_values.iter().chain(resolved_values.iter()).fold(0.0_f64, |a, &b| a.max(b));
    let tickets_chart = create_chart(
        labels.clone(),
        vec![
            create_series("Created", "blue", &created_values, tickets_max, ""),
            create_series("Resolved", "green", &resolved_values, tickets_max, ""),
        ],
        "",
    );

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
