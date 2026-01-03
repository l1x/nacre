use axum::extract::State;
use std::collections::HashMap;
use tracing::debug;

use crate::beads;
use crate::templates::*;

enum MetricsData {
    Issues(beads::Result<Vec<beads::Issue>>),
    Activities(beads::Result<Vec<beads::Activity>>),
    Summary(beads::Result<serde_json::Value>),
}

pub async fn metrics_handler(
    State(state): State<crate::SharedAppState>,
) -> crate::AppResult<MetricsTemplate> {
    // Run all 3 CLI calls in parallel using JoinSet with spawn_blocking
    let mut set: tokio::task::JoinSet<MetricsData> = tokio::task::JoinSet::new();

    let client = state.client.clone();
    set.spawn_blocking(move || MetricsData::Issues(client.list_issues()));

    let client = state.client.clone();
    set.spawn_blocking(move || MetricsData::Activities(client.get_activity()));

    let client = state.client.clone();
    set.spawn_blocking(move || MetricsData::Summary(client.get_status_summary()));

    let mut all_issues = Vec::new();
    let mut activities = Vec::new();
    let mut summary = serde_json::Value::Null;

    while let Some(res) = set.join_next().await {
        match res.map_err(|e| crate::AppError::BadRequest(format!("Task join failed: {e}")))? {
            MetricsData::Issues(data) => all_issues = data?,
            MetricsData::Activities(data) => {
                match data {
                    Ok(acts) => activities = acts,
                    Err(e) => {
                        debug!(error = %e, "Failed to fetch activities");
                        activities = Vec::new();
                    }
                }
            }
            MetricsData::Summary(data) => summary = data.unwrap_or(serde_json::Value::Null),
        }
    }

    let avg_lead_time_hours = summary["summary"]["average_lead_time_hours"]
        .as_f64()
        .unwrap_or(0.0);

    // Calculate Cycle Time
    // Map issue_id to first in_progress timestamp
    let mut started_times: HashMap<String, time::OffsetDateTime> = HashMap::new();
    for act in &activities {
        if act.new_status == Some(beads::Status::InProgress) {
            started_times
                .entry(act.issue_id.clone())
                .or_insert(act.timestamp);
        }
    }
    debug!(
        total_activities = activities.len(),
        in_progress_count = started_times.len(),
        "Cycle time: parsed activities"
    );

    let mut cycle_times = Vec::new();
    let now = time::OffsetDateTime::now_utc();
    let seven_days_ago = now - time::Duration::days(7);
    let mut closed_last_7_days = 0;

    let mut closed_issues_count = 0;
    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at {
            closed_issues_count += 1;
            if closed_at >= seven_days_ago {
                closed_last_7_days += 1;
            }

            // Only include issues that transitioned to InProgress for cycle time
            if let Some(started_at) = started_times.get(&issue.id) {
                let duration = closed_at - *started_at;
                cycle_times.push(duration.whole_minutes() as f64);
            }
        }
    }
    debug!(
        total_issues = all_issues.len(),
        closed_issues = closed_issues_count,
        closed_last_7_days = closed_last_7_days,
        cycle_times_count = cycle_times.len(),
        "Cycle time: matched issues"
    );

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
                .map(|closed| (closed - i.created_at).whole_minutes() as f64 / 60.0)
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

    // Build chart data for the last 7 days
    let now_dt = time::OffsetDateTime::now_utc();
    let start_dt = now_dt - time::Duration::days(6); // 7 days including today

    // Collect all dates
    let mut dates: Vec<time::Date> = Vec::new();
    let mut curr = start_dt.date();
    while curr <= now_dt.date() {
        dates.push(curr);
        if let Some(next) = curr.next_day() {
            curr = next;
        } else {
            break;
        }
    }
    let date_format = time::format_description::parse("[month].[day]").unwrap();
    let labels: Vec<String> = dates
        .iter()
        .map(|d| d.format(&date_format).unwrap())
        .collect();

    // --- Tickets Activity Chart ---
    let mut created_by_day: HashMap<time::Date, usize> = HashMap::new();
    let mut resolved_by_day: HashMap<time::Date, usize> = HashMap::new();
    for d in &dates {
        created_by_day.insert(*d, 0);
        resolved_by_day.insert(*d, 0);
    }
    for issue in &all_issues {
        let created_date = issue.created_at.date();
        if created_date >= start_dt.date() && created_date <= now_dt.date() {
            *created_by_day.entry(created_date).or_insert(0) += 1;
        }
        if let Some(closed_at) = issue.closed_at {
            let resolved_date = closed_at.date();
            if resolved_date >= start_dt.date() && resolved_date <= now_dt.date() {
                *resolved_by_day.entry(resolved_date).or_insert(0) += 1;
            }
        }
    }
    let created_values: Vec<f64> = dates
        .iter()
        .map(|d| *created_by_day.get(d).unwrap_or(&0) as f64)
        .collect();
    let resolved_values: Vec<f64> = dates
        .iter()
        .map(|d| *resolved_by_day.get(d).unwrap_or(&0) as f64)
        .collect();
    let tickets_max = created_values
        .iter()
        .chain(resolved_values.iter())
        .fold(0.0_f64, |a, &b| a.max(b));
    let tickets_chart = create_chart(
        labels.clone(),
        vec![
            create_series("Created", "orange", &created_values, tickets_max, ""),
            create_series("Resolved", "yellow", &resolved_values, tickets_max, ""),
        ],
        "",
    );

    // --- Lead Time Chart (p50, p90, p100 per day) ---
    let mut lead_times_by_day: HashMap<time::Date, Vec<f64>> = HashMap::new();
    for d in &dates {
        lead_times_by_day.insert(*d, Vec::new());
    }
    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at {
            let close_date = closed_at.date();
            if close_date >= start_dt.date() && close_date <= now_dt.date() {
                let lead_time_hours = (closed_at - issue.created_at).whole_minutes() as f64 / 60.0;
                lead_times_by_day
                    .entry(close_date)
                    .or_default()
                    .push(lead_time_hours);
            }
        }
    }
    let (lead_p50, lead_p90, lead_p100): (Vec<f64>, Vec<f64>, Vec<f64>) = dates
        .iter()
        .map(|d| {
            let mut times = lead_times_by_day.get(d).cloned().unwrap_or_default();
            times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            (
                calculate_percentile(&times, 50.0),
                calculate_percentile(&times, 90.0),
                calculate_percentile(&times, 100.0),
            )
        })
        .fold(
            (vec![], vec![], vec![]),
            |(mut a, mut b, mut c), (p50, p90, p100)| {
                a.push(p50);
                b.push(p90);
                c.push(p100);
                (a, b, c)
            },
        );
    let lead_max = lead_p100.iter().fold(0.0_f64, |a, &b| a.max(b));
    let lead_time_chart = create_chart(
        labels.clone(),
        vec![
            create_series("p50", "blue", &lead_p50, lead_max, "h"),
            create_series("p90", "orange", &lead_p90, lead_max, "h"),
            create_series("p100", "yellow", &lead_p100, lead_max, "h"),
        ],
        "h",
    );

    // --- Cycle Time Chart (p50, p90, p100 per day) ---
    let mut cycle_times_by_day: HashMap<time::Date, Vec<f64>> = HashMap::new();
    for d in &dates {
        cycle_times_by_day.insert(*d, Vec::new());
    }
    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at {
            let close_date = closed_at.date();
            if close_date >= start_dt.date() && close_date <= now_dt.date() {
                // Only include issues that transitioned to InProgress for cycle time
                if let Some(started_at) = started_times.get(&issue.id) {
                    let duration_mins = (closed_at - *started_at).whole_minutes() as f64;
                    cycle_times_by_day
                        .entry(close_date)
                        .or_default()
                        .push(duration_mins);
                }
            }
        }
    }
    let (cycle_p50, cycle_p90, cycle_p100): (Vec<f64>, Vec<f64>, Vec<f64>) = dates
        .iter()
        .map(|d| {
            let mut times = cycle_times_by_day.get(d).cloned().unwrap_or_default();
            times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            (
                calculate_percentile(&times, 50.0),
                calculate_percentile(&times, 90.0),
                calculate_percentile(&times, 100.0),
            )
        })
        .fold(
            (vec![], vec![], vec![]),
            |(mut a, mut b, mut c), (p50, p90, p100)| {
                a.push(p50);
                b.push(p90);
                c.push(p100);
                (a, b, c)
            },
        );
    let cycle_max = cycle_p100.iter().fold(0.0_f64, |a, &b| a.max(b));

    let (cycle_unit, cycle_divisor) = if cycle_max > 60.0 {
        ("h", 60.0)
    } else {
        ("m", 1.0)
    };

    let cycle_p50_scaled: Vec<f64> = cycle_p50.iter().map(|v| v / cycle_divisor).collect();
    let cycle_p90_scaled: Vec<f64> = cycle_p90.iter().map(|v| v / cycle_divisor).collect();
    let cycle_p100_scaled: Vec<f64> = cycle_p100.iter().map(|v| v / cycle_divisor).collect();
    let cycle_max_scaled = cycle_max / cycle_divisor;

    let cycle_time_chart = create_chart(
        labels.clone(),
        vec![
            create_series(
                "p50",
                "blue",
                &cycle_p50_scaled,
                cycle_max_scaled,
                cycle_unit,
            ),
            create_series(
                "p90",
                "orange",
                &cycle_p90_scaled,
                cycle_max_scaled,
                cycle_unit,
            ),
            create_series(
                "p100",
                "yellow",
                &cycle_p100_scaled,
                cycle_max_scaled,
                cycle_unit,
            ),
        ],
        cycle_unit,
    );

    // --- Throughput Chart ---
    let throughput_values: Vec<f64> = dates
        .iter()
        .map(|d| *resolved_by_day.get(d).unwrap_or(&0) as f64)
        .collect();
    let throughput_max = throughput_values.iter().fold(0.0_f64, |a, &b| a.max(b));
    let throughput_chart = create_chart(
        labels,
        vec![create_series(
            "Closed",
            "orange",
            &throughput_values,
            throughput_max,
            "",
        )],
        "",
    );

    // --- Activity Heat Map (day of week × hour of day) ---
    // Grid: 7 days (rows) × 24 hours (cols)
    let mut heatmap_grid: [[usize; 24]; 7] = [[0; 24]; 7];

    // Count activity events by day of week and hour
    for activity in &activities {
        let hour = activity.timestamp.hour() as usize;
        let weekday = activity.timestamp.weekday().number_days_from_monday() as usize;
        heatmap_grid[weekday][hour] += 1;
    }

    // Also count issue creation times
    for issue in &all_issues {
        let hour = issue.created_at.hour() as usize;
        let weekday = issue.created_at.weekday().number_days_from_monday() as usize;
        heatmap_grid[weekday][hour] += 1;
    }

    let heatmap_max = heatmap_grid
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .max()
        .unwrap_or(0);

    // Convert to HeatMapData with intensity levels (0-4)
    // Row labels: days of week (Y-axis)
    let row_labels: Vec<String> = vec![
        "Mon".to_string(),
        "Tue".to_string(),
        "Wed".to_string(),
        "Thu".to_string(),
        "Fri".to_string(),
        "Sat".to_string(),
        "Sun".to_string(),
    ];
    // Column labels: hours (X-axis)
    let col_labels: Vec<String> = (0..24).map(|h| format!("{:02}", h)).collect();

    let cells: Vec<Vec<HeatMapCell>> = heatmap_grid
        .iter()
        .map(|row| {
            row.iter()
                .map(|&value| {
                    let intensity = if heatmap_max == 0 {
                        0
                    } else {
                        // Scale to 0-4 intensity levels
                        ((value as f64 / heatmap_max as f64) * 4.0).ceil() as u8
                    };
                    HeatMapCell { value, intensity }
                })
                .collect()
        })
        .collect();

    let activity_heatmap = HeatMapData {
        row_labels,
        col_labels,
        cells,
        max_value: heatmap_max,
    };

    Ok(MetricsTemplate {
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
        tickets_chart,
        lead_time_chart,
        cycle_time_chart,
        throughput_chart,
        p50_lead_time_hours,
        p90_lead_time_hours,
        p100_lead_time_hours,
        p50_cycle_time_mins,
        p90_cycle_time_mins,
        p100_cycle_time_mins,
        activity_heatmap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::{Activity, EventType, Issue, Status};

    #[test]
    fn test_cycle_time_calculation_with_in_progress() {
        // Create test activities with InProgress transition
        let activities = vec![
            Activity {
                timestamp: time::macros::datetime!(2026-01-01 10:00:00 UTC),
                r#type: EventType::StatusChanged,
                issue_id: "test-1".to_string(),
                message: "started".to_string(),
                old_status: Some(Status::Open),
                new_status: Some(Status::InProgress),
            },
            Activity {
                timestamp: time::macros::datetime!(2026-01-01 12:00:00 UTC),
                r#type: EventType::StatusChanged,
                issue_id: "test-1".to_string(),
                message: "completed".to_string(),
                old_status: Some(Status::InProgress),
                new_status: Some(Status::Closed),
            },
        ];

        // Build started_times map (same logic as in metrics_handler)
        let mut started_times: HashMap<String, time::OffsetDateTime> = HashMap::new();
        for act in &activities {
            if act.new_status == Some(Status::InProgress) {
                started_times
                    .entry(act.issue_id.clone())
                    .or_insert(act.timestamp);
            }
        }

        // Should have one entry
        assert_eq!(started_times.len(), 1);
        assert!(started_times.contains_key("test-1"));

        // Create a closed issue
        let issue = Issue {
            id: "test-1".to_string(),
            title: "Test Issue".to_string(),
            status: Status::Closed,
            priority: Some(2),
            issue_type: crate::beads::IssueType::Task,
            created_at: time::macros::datetime!(2026-01-01 09:00:00 UTC),
            updated_at: time::macros::datetime!(2026-01-01 12:00:00 UTC),
            closed_at: Some(time::macros::datetime!(2026-01-01 12:00:00 UTC)),
            assignee: None,
            labels: None,
            description: None,
            acceptance_criteria: None,
            close_reason: None,
            estimate: None,
            dependencies: vec![],
        };

        // Calculate cycle time (same logic as in metrics_handler)
        let mut cycle_times = Vec::new();
        if let Some(closed_at) = issue.closed_at {
            if let Some(started_at) = started_times.get(&issue.id) {
                let duration = closed_at - *started_at;
                cycle_times.push(duration.whole_minutes() as f64);
            }
        }

        // Should have cycle time of 120 minutes (2 hours)
        assert_eq!(cycle_times.len(), 1);
        assert_eq!(cycle_times[0], 120.0);
    }

    #[test]
    fn test_cycle_time_empty_without_in_progress() {
        // Create activities WITHOUT InProgress transition
        let activities = vec![
            Activity {
                timestamp: time::macros::datetime!(2026-01-01 10:00:00 UTC),
                r#type: EventType::Created,
                issue_id: "test-1".to_string(),
                message: "created".to_string(),
                old_status: None,
                new_status: Some(Status::Open),
            },
        ];

        // Build started_times map
        let mut started_times: HashMap<String, time::OffsetDateTime> = HashMap::new();
        for act in &activities {
            if act.new_status == Some(Status::InProgress) {
                started_times
                    .entry(act.issue_id.clone())
                    .or_insert(act.timestamp);
            }
        }

        // Should be empty - no InProgress transitions
        assert_eq!(started_times.len(), 0);
    }
}
