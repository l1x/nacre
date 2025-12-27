use axum::extract::State;
use std::collections::HashMap;

use crate::beads;
use crate::templates::*;

pub async fn metrics_handler(State(state): State<crate::AppState>) -> MetricsTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();
    let activities = state.client.get_activity().unwrap_or_default();
    let summary = state.client.get_status_summary().unwrap_or_default();

    let avg_lead_time_hours = summary["summary"]["average_lead_time_hours"]
        .as_f64()
        .unwrap_or(0.0);

    // Calculate Cycle Time
    // Map issue_id to first in_progress timestamp
    let mut started_times: HashMap<String, chrono::DateTime<chrono::FixedOffset>> = HashMap::new();
    for act in &activities {
        if act.new_status == Some(beads::Status::InProgress) {
            started_times
                .entry(act.issue_id.clone())
                .or_insert(act.timestamp);
        }
    }

    let mut cycle_times = Vec::new();
    let now = chrono::Utc::now();
    let seven_days_ago = now - chrono::Duration::days(7);
    let mut closed_last_7_days = 0;

    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at {
            if closed_at.with_timezone(&chrono::Utc) >= seven_days_ago {
                closed_last_7_days += 1;
            }

            if let Some(started_at) = started_times.get(&issue.id) {
                let duration = closed_at - *started_at;
                cycle_times.push(duration.num_minutes() as f64);
            }
        }
    }

    let avg_cycle_time_mins = if !cycle_times.is_empty() {
        cycle_times.iter().sum::<f64>() / cycle_times.len() as f64
    } else {
        0.0
    };

    let mut sorted_cycle_times = cycle_times.clone();
    sorted_cycle_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let throughput_per_day = closed_last_7_days as f64 / 7.0;

    let wip_count = all_issues
        .iter()
        .filter(|i| i.status == beads::Status::InProgress)
        .count();
    let blocked_count = all_issues
        .iter()
        .filter(|i| i.status == beads::Status::Blocked)
        .count();

    // Calculate global percentiles for Lead Time
    let mut all_lead_times: Vec<f64> = all_issues
        .iter()
        .filter_map(|i| {
            i.closed_at
                .map(|closed| (closed - i.created_at).num_minutes() as f64 / 60.0)
        })
        .collect();
    all_lead_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    fn calculate_percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = ((sorted.len() as f64 - 1.0) * p / 100.0).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    let p50_lead_time_hours = calculate_percentile(&all_lead_times, 50.0);
    let p90_lead_time_hours = calculate_percentile(&all_lead_times, 90.0);
    let p100_lead_time_hours = calculate_percentile(&all_lead_times, 100.0);

    let p50_cycle_time_mins = calculate_percentile(&sorted_cycle_times, 50.0);
    let p90_cycle_time_mins = calculate_percentile(&sorted_cycle_times, 90.0);
    let p100_cycle_time_mins = calculate_percentile(&sorted_cycle_times, 100.0);

    // Chart data will be passed to templates in the next step
    // For now, use empty strings as placeholders
    MetricsTemplate {
        project_name: state.project_name.clone(),
        page_title: "Metrics".to_string(),
        active_nav: "metrics",
        app_version: state.app_version.clone(),
        avg_lead_time_hours,
        avg_cycle_time_mins,
        throughput_per_day,
        closed_last_7_days,
        wip_count,
        blocked_count,
        tickets_chart_svg: String::new(),
        lead_time_chart_svg: String::new(),
        cycle_time_distribution_svg: String::new(),
        throughput_distribution_svg: String::new(),
        p50_lead_time_hours,
        p90_lead_time_hours,
        p100_lead_time_hours,
        p50_cycle_time_mins,
        p90_cycle_time_mins,
        p100_cycle_time_mins,
    }
}
