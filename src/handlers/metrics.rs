use axum::extract::State;
use std::collections::HashMap;

use crate::beads;
use crate::templates::*;

/// Helper to create a series of bars from values
fn create_series(name: &str, color: &'static str, values: &[f64], max: f64, unit: &str) -> ChartSeries {
    ChartSeries {
        name: name.to_string(),
        color,
        bars: values
            .iter()
            .map(|&v| ChartBar {
                value: v,
                percent: if max > 0.0 { (v / max) * 100.0 } else { 0.0 },
                display: if unit.is_empty() {
                    format!("{}", v as i64)
                } else {
                    format!("{:.1}{}", v, unit)
                },
            })
            .collect(),
    }
}

/// Helper to create chart data from multiple series
fn create_chart(labels: Vec<String>, series: Vec<ChartSeries>, unit: &'static str) -> ChartData {
    let max_value = series
        .iter()
        .flat_map(|s| s.bars.iter().map(|b| b.value))
        .fold(0.0_f64, |a, b| a.max(b));

    ChartData {
        labels,
        series,
        unit,
        max_value,
    }
}

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

            let started_at = started_times.get(&issue.id).unwrap_or(&issue.created_at);
            let duration = closed_at - *started_at;
            cycle_times.push(duration.num_minutes() as f64);
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

    // Build chart data for the last 7 days
    let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let start_dt = now_dt - chrono::Duration::days(6); // 7 days including today

    // Collect all dates
    let mut dates: Vec<chrono::NaiveDate> = Vec::new();
    let mut curr = start_dt.date_naive();
    while curr <= now_dt.date_naive() {
        dates.push(curr);
        curr = curr.succ_opt().unwrap();
    }
    let labels: Vec<String> = dates.iter().map(|d| d.format("%m.%d").to_string()).collect();

    // --- Tickets Activity Chart ---
    let mut created_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();
    let mut resolved_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();
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

    // --- Lead Time Chart (p50, p90, p100 per day) ---
    let mut lead_times_by_day: HashMap<chrono::NaiveDate, Vec<f64>> = HashMap::new();
    for d in &dates {
        lead_times_by_day.insert(*d, Vec::new());
    }
    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at {
            let close_date = closed_at.date_naive();
            if close_date >= start_dt.date_naive() && close_date <= now_dt.date_naive() {
                let lead_time_hours = (closed_at - issue.created_at).num_minutes() as f64 / 60.0;
                lead_times_by_day.entry(close_date).or_default().push(lead_time_hours);
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
        .fold((vec![], vec![], vec![]), |(mut a, mut b, mut c), (p50, p90, p100)| {
            a.push(p50);
            b.push(p90);
            c.push(p100);
            (a, b, c)
        });
    let lead_max = lead_p100.iter().fold(0.0_f64, |a, &b| a.max(b));
    let lead_time_chart = create_chart(
        labels.clone(),
        vec![
            create_series("p50", "blue", &lead_p50, lead_max, "h"),
            create_series("p90", "green", &lead_p90, lead_max, "h"),
            create_series("p100", "orange", &lead_p100, lead_max, "h"),
        ],
        "h",
    );

    // --- Cycle Time Chart (p50, p90, p100 per day) ---
    let mut cycle_times_by_day: HashMap<chrono::NaiveDate, Vec<f64>> = HashMap::new();
    for d in &dates {
        cycle_times_by_day.insert(*d, Vec::new());
    }
    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at {
            let close_date = closed_at.date_naive();
            if close_date >= start_dt.date_naive() && close_date <= now_dt.date_naive() {
                let started_at = started_times.get(&issue.id).unwrap_or(&issue.created_at);
                let duration_mins = (closed_at - *started_at).num_minutes() as f64;
                cycle_times_by_day.entry(close_date).or_default().push(duration_mins);
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
        .fold((vec![], vec![], vec![]), |(mut a, mut b, mut c), (p50, p90, p100)| {
            a.push(p50);
            b.push(p90);
            c.push(p100);
            (a, b, c)
        });
    let cycle_max = cycle_p100.iter().fold(0.0_f64, |a, &b| a.max(b));
    let cycle_time_chart = create_chart(
        labels.clone(),
        vec![
            create_series("p50", "blue", &cycle_p50, cycle_max, "m"),
            create_series("p90", "green", &cycle_p90, cycle_max, "m"),
            create_series("p100", "orange", &cycle_p100, cycle_max, "m"),
        ],
        "m",
    );

    // --- Throughput Chart ---
    let throughput_values: Vec<f64> = dates
        .iter()
        .map(|d| *resolved_by_day.get(d).unwrap_or(&0) as f64)
        .collect();
    let throughput_max = throughput_values.iter().fold(0.0_f64, |a, &b| a.max(b));
    let throughput_chart = create_chart(
        labels,
        vec![create_series("Closed", "blue", &throughput_values, throughput_max, "")],
        "",
    );

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
    }
}
