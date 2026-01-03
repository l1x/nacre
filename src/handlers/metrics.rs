use axum::extract::State;
use std::collections::HashMap;
use time::OffsetDateTime;
use tracing::debug;

use crate::beads::{self, Activity, Issue, Status};
use crate::templates::*;

// ============================================================================
// Pure Functions - testable without mocking
// ============================================================================

/// Calculate percentile from a sorted slice of values
pub fn calculate_percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p / 100.0).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Build a map of issue_id -> first in_progress timestamp
pub fn build_started_times_map(activities: &[Activity]) -> HashMap<String, OffsetDateTime> {
    let mut started_times: HashMap<String, OffsetDateTime> = HashMap::new();
    for act in activities {
        if act.new_status == Some(Status::InProgress) {
            started_times
                .entry(act.issue_id.clone())
                .or_insert(act.timestamp);
        }
    }
    started_times
}

/// Cycle time statistics
pub struct CycleTimeStats {
    pub avg_mins: f64,
    pub p50_mins: f64,
    pub p90_mins: f64,
    pub p100_mins: f64,
    pub count: usize,
}

/// Calculate cycle times from issues and their started times
pub fn calculate_cycle_times(
    issues: &[Issue],
    started_times: &HashMap<String, OffsetDateTime>,
) -> CycleTimeStats {
    let mut cycle_times: Vec<f64> = Vec::new();

    for issue in issues {
        if let Some(closed_at) = issue.closed_at
            && let Some(started_at) = started_times.get(&issue.id)
        {
            let duration = closed_at - *started_at;
            cycle_times.push(duration.whole_minutes() as f64);
        }
    }

    let avg_mins = if !cycle_times.is_empty() {
        cycle_times.iter().sum::<f64>() / cycle_times.len() as f64
    } else {
        0.0
    };

    cycle_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    CycleTimeStats {
        avg_mins,
        p50_mins: calculate_percentile(&cycle_times, 50.0),
        p90_mins: calculate_percentile(&cycle_times, 90.0),
        p100_mins: calculate_percentile(&cycle_times, 100.0),
        count: cycle_times.len(),
    }
}

/// Lead time statistics
pub struct LeadTimeStats {
    pub avg_hours: f64,
    pub p50_hours: f64,
    pub p90_hours: f64,
    pub p100_hours: f64,
}

/// Calculate lead times from issues (created_at to closed_at)
pub fn calculate_lead_times(issues: &[Issue]) -> LeadTimeStats {
    let mut lead_times: Vec<f64> = issues
        .iter()
        .filter_map(|i| {
            i.closed_at
                .map(|closed| (closed - i.created_at).whole_minutes() as f64 / 60.0)
        })
        .collect();

    let avg_hours = if !lead_times.is_empty() {
        lead_times.iter().sum::<f64>() / lead_times.len() as f64
    } else {
        0.0
    };

    lead_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    LeadTimeStats {
        avg_hours,
        p50_hours: calculate_percentile(&lead_times, 50.0),
        p90_hours: calculate_percentile(&lead_times, 90.0),
        p100_hours: calculate_percentile(&lead_times, 100.0),
    }
}

/// Count issues by status
pub struct StatusCounts {
    pub closed_last_7_days: usize,
    pub wip_count: usize,
    pub blocked_count: usize,
}

/// Calculate status counts from issues
pub fn calculate_status_counts(issues: &[Issue], now: OffsetDateTime) -> StatusCounts {
    let seven_days_ago = now - time::Duration::days(7);

    let closed_last_7_days = issues
        .iter()
        .filter(|i| i.closed_at.is_some_and(|c| c >= seven_days_ago))
        .count();

    let wip_count = issues
        .iter()
        .filter(|i| i.status == Status::InProgress)
        .count();

    let blocked_count = issues.iter().filter(|i| i.status == Status::Blocked).count();

    StatusCounts {
        closed_last_7_days,
        wip_count,
        blocked_count,
    }
}

/// Generate date range for chart data (last 7 days including today)
pub fn generate_date_range(now: OffsetDateTime) -> Vec<time::Date> {
    let start_dt = now - time::Duration::days(6);
    let mut dates: Vec<time::Date> = Vec::new();
    let mut curr = start_dt.date();
    while curr <= now.date() {
        dates.push(curr);
        if let Some(next) = curr.next_day() {
            curr = next;
        } else {
            break;
        }
    }
    dates
}

/// Format dates as labels for charts
pub fn format_date_labels(dates: &[time::Date]) -> Vec<String> {
    let date_format = time::format_description::parse("[month].[day]").unwrap();
    dates
        .iter()
        .map(|d| d.format(&date_format).unwrap())
        .collect()
}

/// Build tickets activity chart (created vs resolved per day)
pub fn build_tickets_chart(issues: &[Issue], dates: &[time::Date]) -> ChartData {
    let start_date = dates.first().copied().unwrap_or(time::Date::MIN);
    let end_date = dates.last().copied().unwrap_or(time::Date::MAX);

    let mut created_by_day: HashMap<time::Date, usize> = HashMap::new();
    let mut resolved_by_day: HashMap<time::Date, usize> = HashMap::new();

    for d in dates {
        created_by_day.insert(*d, 0);
        resolved_by_day.insert(*d, 0);
    }

    for issue in issues {
        let created_date = issue.created_at.date();
        if created_date >= start_date && created_date <= end_date {
            *created_by_day.entry(created_date).or_insert(0) += 1;
        }
        if let Some(closed_at) = issue.closed_at {
            let resolved_date = closed_at.date();
            if resolved_date >= start_date && resolved_date <= end_date {
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

    let max_val = created_values
        .iter()
        .chain(resolved_values.iter())
        .fold(0.0_f64, |a, &b| a.max(b));

    let labels = format_date_labels(dates);

    create_chart(
        labels,
        vec![
            create_series("Created", "orange", &created_values, max_val, ""),
            create_series("Resolved", "yellow", &resolved_values, max_val, ""),
        ],
        "",
    )
}

/// Build lead time chart with p50, p90, p100 per day
pub fn build_lead_time_chart(issues: &[Issue], dates: &[time::Date]) -> ChartData {
    let start_date = dates.first().copied().unwrap_or(time::Date::MIN);
    let end_date = dates.last().copied().unwrap_or(time::Date::MAX);

    let mut lead_times_by_day: HashMap<time::Date, Vec<f64>> = HashMap::new();
    for d in dates {
        lead_times_by_day.insert(*d, Vec::new());
    }

    for issue in issues {
        if let Some(closed_at) = issue.closed_at {
            let close_date = closed_at.date();
            if close_date >= start_date && close_date <= end_date {
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
    let labels = format_date_labels(dates);

    create_chart(
        labels,
        vec![
            create_series("p50", "blue", &lead_p50, lead_max, "h"),
            create_series("p90", "orange", &lead_p90, lead_max, "h"),
            create_series("p100", "yellow", &lead_p100, lead_max, "h"),
        ],
        "h",
    )
}

/// Build cycle time chart with p50, p90, p100 per day
pub fn build_cycle_time_chart(
    issues: &[Issue],
    started_times: &HashMap<String, OffsetDateTime>,
    dates: &[time::Date],
) -> ChartData {
    let start_date = dates.first().copied().unwrap_or(time::Date::MIN);
    let end_date = dates.last().copied().unwrap_or(time::Date::MAX);

    let mut cycle_times_by_day: HashMap<time::Date, Vec<f64>> = HashMap::new();
    for d in dates {
        cycle_times_by_day.insert(*d, Vec::new());
    }

    for issue in issues {
        if let Some(closed_at) = issue.closed_at {
            let close_date = closed_at.date();
            if close_date >= start_date
                && close_date <= end_date
                && let Some(started_at) = started_times.get(&issue.id)
            {
                let duration_mins = (closed_at - *started_at).whole_minutes() as f64;
                cycle_times_by_day
                    .entry(close_date)
                    .or_default()
                    .push(duration_mins);
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

    let labels = format_date_labels(dates);

    create_chart(
        labels,
        vec![
            create_series("p50", "blue", &cycle_p50_scaled, cycle_max_scaled, cycle_unit),
            create_series("p90", "orange", &cycle_p90_scaled, cycle_max_scaled, cycle_unit),
            create_series("p100", "yellow", &cycle_p100_scaled, cycle_max_scaled, cycle_unit),
        ],
        cycle_unit,
    )
}

/// Build throughput chart (closed issues per day)
pub fn build_throughput_chart(issues: &[Issue], dates: &[time::Date]) -> ChartData {
    let start_date = dates.first().copied().unwrap_or(time::Date::MIN);
    let end_date = dates.last().copied().unwrap_or(time::Date::MAX);

    let mut resolved_by_day: HashMap<time::Date, usize> = HashMap::new();
    for d in dates {
        resolved_by_day.insert(*d, 0);
    }

    for issue in issues {
        if let Some(closed_at) = issue.closed_at {
            let resolved_date = closed_at.date();
            if resolved_date >= start_date && resolved_date <= end_date {
                *resolved_by_day.entry(resolved_date).or_insert(0) += 1;
            }
        }
    }

    let throughput_values: Vec<f64> = dates
        .iter()
        .map(|d| *resolved_by_day.get(d).unwrap_or(&0) as f64)
        .collect();

    let throughput_max = throughput_values.iter().fold(0.0_f64, |a, &b| a.max(b));
    let labels = format_date_labels(dates);

    create_chart(
        labels,
        vec![create_series("Closed", "orange", &throughput_values, throughput_max, "")],
        "",
    )
}

/// Build activity heat map (day of week × hour of day)
pub fn build_activity_heatmap(activities: &[Activity], issues: &[Issue]) -> HeatMapData {
    let mut heatmap_grid: [[usize; 24]; 7] = [[0; 24]; 7];

    // Count activity events by day of week and hour
    for activity in activities {
        let hour = activity.timestamp.hour() as usize;
        let weekday = activity.timestamp.weekday().number_days_from_monday() as usize;
        heatmap_grid[weekday][hour] += 1;
    }

    // Also count issue creation times
    for issue in issues {
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

    let row_labels: Vec<String> = vec![
        "Mon".to_string(),
        "Tue".to_string(),
        "Wed".to_string(),
        "Thu".to_string(),
        "Fri".to_string(),
        "Sat".to_string(),
        "Sun".to_string(),
    ];
    let col_labels: Vec<String> = (0..24).map(|h| format!("{:02}", h)).collect();

    let cells: Vec<Vec<HeatMapCell>> = heatmap_grid
        .iter()
        .map(|row| {
            row.iter()
                .map(|&value| {
                    let intensity = if heatmap_max == 0 {
                        0
                    } else {
                        ((value as f64 / heatmap_max as f64) * 4.0).ceil() as u8
                    };
                    HeatMapCell { value, intensity }
                })
                .collect()
        })
        .collect();

    HeatMapData {
        row_labels,
        col_labels,
        cells,
        max_value: heatmap_max,
    }
}

// ============================================================================
// Handler - thin orchestration layer
// ============================================================================

enum MetricsData {
    Issues(beads::Result<Vec<Issue>>),
    Activities(beads::Result<Vec<Activity>>),
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

    // Use pure functions for all calculations
    let now = OffsetDateTime::now_utc();
    let dates = generate_date_range(now);

    let started_times = build_started_times_map(&activities);
    debug!(
        total_activities = activities.len(),
        in_progress_count = started_times.len(),
        "Cycle time: parsed activities"
    );

    let cycle_stats = calculate_cycle_times(&all_issues, &started_times);
    let lead_stats = calculate_lead_times(&all_issues);
    let status_counts = calculate_status_counts(&all_issues, now);

    debug!(
        total_issues = all_issues.len(),
        closed_last_7_days = status_counts.closed_last_7_days,
        cycle_times_count = cycle_stats.count,
        "Cycle time: matched issues"
    );

    let avg_lead_time_hours = summary["summary"]["average_lead_time_hours"]
        .as_f64()
        .unwrap_or(lead_stats.avg_hours);

    let throughput_per_day = status_counts.closed_last_7_days as f64 / 7.0;

    // Build charts using pure functions
    let tickets_chart = build_tickets_chart(&all_issues, &dates);
    let lead_time_chart = build_lead_time_chart(&all_issues, &dates);
    let cycle_time_chart = build_cycle_time_chart(&all_issues, &started_times, &dates);
    let throughput_chart = build_throughput_chart(&all_issues, &dates);
    let activity_heatmap = build_activity_heatmap(&activities, &all_issues);

    Ok(MetricsTemplate {
        project_name: state.project_name.clone(),
        page_title: "Metrics".to_string(),
        active_nav: "metrics",
        app_version: state.app_version.clone(),
        avg_lead_time_hours,
        avg_cycle_time_mins: cycle_stats.avg_mins,
        throughput_per_day,
        closed_last_7_days: status_counts.closed_last_7_days,
        wip_count: status_counts.wip_count,
        blocked_count: status_counts.blocked_count,
        tickets_chart,
        lead_time_chart,
        cycle_time_chart,
        throughput_chart,
        p50_lead_time_hours: lead_stats.p50_hours,
        p90_lead_time_hours: lead_stats.p90_hours,
        p100_lead_time_hours: lead_stats.p100_hours,
        p50_cycle_time_mins: cycle_stats.p50_mins,
        p90_cycle_time_mins: cycle_stats.p90_mins,
        p100_cycle_time_mins: cycle_stats.p100_mins,
        activity_heatmap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beads::{Activity, EventType, IssueType};

    fn make_test_issue(id: &str, status: Status, created_at: OffsetDateTime, closed_at: Option<OffsetDateTime>) -> Issue {
        Issue {
            id: id.to_string(),
            title: format!("Test {}", id),
            status,
            priority: Some(2),
            issue_type: IssueType::Task,
            created_at,
            updated_at: closed_at.unwrap_or(created_at),
            closed_at,
            assignee: None,
            labels: None,
            description: None,
            acceptance_criteria: None,
            close_reason: None,
            estimate: None,
            dependencies: vec![],
        }
    }

    fn make_status_change(issue_id: &str, timestamp: OffsetDateTime, new_status: Status) -> Activity {
        Activity {
            timestamp,
            r#type: EventType::StatusChanged,
            issue_id: issue_id.to_string(),
            message: "status changed".to_string(),
            old_status: Some(Status::Open),
            new_status: Some(new_status),
        }
    }

    #[test]
    fn test_calculate_percentile_empty() {
        assert_eq!(calculate_percentile(&[], 50.0), 0.0);
    }

    #[test]
    fn test_calculate_percentile_single() {
        assert_eq!(calculate_percentile(&[42.0], 50.0), 42.0);
        assert_eq!(calculate_percentile(&[42.0], 100.0), 42.0);
    }

    #[test]
    fn test_calculate_percentile_multiple() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(calculate_percentile(&values, 0.0), 10.0);
        assert_eq!(calculate_percentile(&values, 50.0), 30.0);
        assert_eq!(calculate_percentile(&values, 100.0), 50.0);
    }

    #[test]
    fn test_build_started_times_map() {
        let activities = vec![
            make_status_change("test-1", time::macros::datetime!(2026-01-01 10:00:00 UTC), Status::InProgress),
            make_status_change("test-1", time::macros::datetime!(2026-01-01 12:00:00 UTC), Status::Closed),
            make_status_change("test-2", time::macros::datetime!(2026-01-01 11:00:00 UTC), Status::InProgress),
        ];

        let started_times = build_started_times_map(&activities);

        assert_eq!(started_times.len(), 2);
        assert!(started_times.contains_key("test-1"));
        assert!(started_times.contains_key("test-2"));
        // Should use first InProgress timestamp
        assert_eq!(started_times["test-1"], time::macros::datetime!(2026-01-01 10:00:00 UTC));
    }

    #[test]
    fn test_build_started_times_map_empty_without_in_progress() {
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

        let started_times = build_started_times_map(&activities);
        assert_eq!(started_times.len(), 0);
    }

    #[test]
    fn test_calculate_cycle_times() {
        let issues = vec![
            make_test_issue(
                "test-1",
                Status::Closed,
                time::macros::datetime!(2026-01-01 09:00:00 UTC),
                Some(time::macros::datetime!(2026-01-01 12:00:00 UTC)),
            ),
        ];

        let mut started_times = HashMap::new();
        started_times.insert(
            "test-1".to_string(),
            time::macros::datetime!(2026-01-01 10:00:00 UTC),
        );

        let stats = calculate_cycle_times(&issues, &started_times);

        assert_eq!(stats.count, 1);
        assert_eq!(stats.avg_mins, 120.0); // 2 hours = 120 minutes
        assert_eq!(stats.p50_mins, 120.0);
    }

    #[test]
    fn test_calculate_cycle_times_no_in_progress() {
        let issues = vec![
            make_test_issue(
                "test-1",
                Status::Closed,
                time::macros::datetime!(2026-01-01 09:00:00 UTC),
                Some(time::macros::datetime!(2026-01-01 12:00:00 UTC)),
            ),
        ];

        let started_times = HashMap::new(); // No InProgress transitions

        let stats = calculate_cycle_times(&issues, &started_times);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.avg_mins, 0.0);
    }

    #[test]
    fn test_calculate_lead_times() {
        let issues = vec![
            make_test_issue(
                "test-1",
                Status::Closed,
                time::macros::datetime!(2026-01-01 09:00:00 UTC),
                Some(time::macros::datetime!(2026-01-01 12:00:00 UTC)),
            ),
            make_test_issue(
                "test-2",
                Status::Closed,
                time::macros::datetime!(2026-01-01 10:00:00 UTC),
                Some(time::macros::datetime!(2026-01-01 14:00:00 UTC)),
            ),
        ];

        let stats = calculate_lead_times(&issues);

        // test-1: 3 hours, test-2: 4 hours, avg = 3.5 hours
        assert_eq!(stats.avg_hours, 3.5);
        // With 2 values [3.0, 4.0], p50 rounds to index 1 = 4.0
        assert_eq!(stats.p50_hours, 4.0);
        assert_eq!(stats.p100_hours, 4.0);
    }

    #[test]
    fn test_calculate_status_counts() {
        let now = time::macros::datetime!(2026-01-05 12:00:00 UTC);
        let issues = vec![
            make_test_issue(
                "test-1",
                Status::Closed,
                time::macros::datetime!(2026-01-01 09:00:00 UTC),
                Some(time::macros::datetime!(2026-01-04 12:00:00 UTC)), // within 7 days
            ),
            make_test_issue(
                "test-2",
                Status::InProgress,
                time::macros::datetime!(2026-01-02 10:00:00 UTC),
                None,
            ),
            make_test_issue(
                "test-3",
                Status::Blocked,
                time::macros::datetime!(2026-01-03 10:00:00 UTC),
                None,
            ),
        ];

        let counts = calculate_status_counts(&issues, now);

        assert_eq!(counts.closed_last_7_days, 1);
        assert_eq!(counts.wip_count, 1);
        assert_eq!(counts.blocked_count, 1);
    }

    #[test]
    fn test_generate_date_range() {
        let now = time::macros::datetime!(2026-01-07 12:00:00 UTC);
        let dates = generate_date_range(now);

        assert_eq!(dates.len(), 7);
        assert_eq!(dates[0], time::macros::date!(2026-01-01));
        assert_eq!(dates[6], time::macros::date!(2026-01-07));
    }

    #[test]
    fn test_build_activity_heatmap() {
        // 2026-01-05 is a Monday
        let activities = vec![
            Activity {
                timestamp: time::macros::datetime!(2026-01-05 14:00:00 UTC), // Monday 14:00
                r#type: EventType::StatusChanged,
                issue_id: "test-1".to_string(),
                message: "changed".to_string(),
                old_status: Some(Status::Open),
                new_status: Some(Status::InProgress),
            },
        ];

        let issues = vec![
            make_test_issue(
                "test-1",
                Status::InProgress,
                time::macros::datetime!(2026-01-05 10:00:00 UTC), // Monday 10:00
                None,
            ),
        ];

        let heatmap = build_activity_heatmap(&activities, &issues);

        assert_eq!(heatmap.row_labels.len(), 7);
        assert_eq!(heatmap.col_labels.len(), 24);
        assert_eq!(heatmap.max_value, 1);
        // Monday (row 0) should have activity at hours 10 and 14
        assert_eq!(heatmap.cells[0][10].value, 1);
        assert_eq!(heatmap.cells[0][14].value, 1);
    }
}
