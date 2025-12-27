use axum::extract::State;
use charts_rs::{BarChart, Color, Series, THEME_DARK};
use std::collections::HashMap;

use crate::beads;
use crate::templates::*;

/// Nacre dark theme background color (#231f1d)
const NACRE_BG: Color = Color {
    r: 35,
    g: 31,
    b: 29,
    a: 255,
};

/// Nacre accent blue (#4f81bd)
const NACRE_BLUE: Color = Color {
    r: 79,
    g: 129,
    b: 189,
    a: 255,
};

/// Nacre accent green (#9bbb59)
const NACRE_GREEN: Color = Color {
    r: 155,
    g: 187,
    b: 89,
    a: 255,
};

/// Nacre accent orange (#f79646)
const NACRE_ORANGE: Color = Color {
    r: 247,
    g: 150,
    b: 70,
    a: 255,
};

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

    // Generate Tickets Activity Chart using charts-rs
    let tickets_chart_svg = {
        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(7);

        let mut created_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();
        let mut resolved_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();

        // Initialize maps with 0 for all days
        let mut curr = start_dt.date_naive();
        while curr <= now_dt.date_naive() {
            created_by_day.insert(curr, 0);
            resolved_by_day.insert(curr, 0);
            curr = curr.succ_opt().unwrap();
        }

        for issue in &all_issues {
            let created_date = issue.created_at.date_naive();
            if created_date >= start_dt.date_naive() {
                *created_by_day.entry(created_date).or_insert(0) += 1;
            }
            if let Some(closed_at) = issue.closed_at {
                let resolved_date = closed_at.date_naive();
                if resolved_date >= start_dt.date_naive() {
                    *resolved_by_day.entry(resolved_date).or_insert(0) += 1;
                }
            }
        }

        // Collect and sort dates
        let mut all_dates: Vec<chrono::NaiveDate> = created_by_day.keys().cloned().collect();
        all_dates.sort();

        let mut created_data: Vec<f32> = Vec::new();
        let mut resolved_data: Vec<f32> = Vec::new();
        let mut x_labels: Vec<String> = Vec::new();

        for date in &all_dates {
            let created = *created_by_day.get(date).unwrap_or(&0);
            let resolved = *resolved_by_day.get(date).unwrap_or(&0);
            created_data.push(created as f32);
            resolved_data.push(resolved as f32);
            x_labels.push(date.format("%a").to_string());
        }

        let mut chart = BarChart::new_with_theme(
            vec![
                Series::new("Created".to_string(), created_data),
                Series::new("Resolved".to_string(), resolved_data),
            ],
            x_labels,
            THEME_DARK,
        );

        chart.width = 700.0;
        chart.height = 400.0;
        chart.background_color = NACRE_BG;
        chart.series_colors = vec![NACRE_BLUE, NACRE_GREEN];
        chart.series_list[0].label_show = true;
        chart.series_list[1].label_show = true;

        chart.svg().unwrap_or_default()
    };

    // Generate Lead Time Percentiles Chart (p50, p90, p100 over time) using charts-rs
    let lead_time_chart_svg = {
        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(7);

        // Group closed issues by close date and calculate lead times
        let mut lead_times_by_day: HashMap<chrono::NaiveDate, Vec<f64>> = HashMap::new();
        for issue in &all_issues {
            if let Some(closed_at) = issue.closed_at {
                let close_date = closed_at.date_naive();
                if close_date >= start_dt.date_naive() {
                    let lead_time_hours = (closed_at - issue.created_at).num_minutes() as f64 / 60.0;
                    lead_times_by_day
                        .entry(close_date)
                        .or_default()
                        .push(lead_time_hours);
                }
            }
        }

        if !lead_times_by_day.is_empty() {
            // Collect and sort dates
            let mut all_dates: Vec<chrono::NaiveDate> = lead_times_by_day.keys().cloned().collect();
            all_dates.sort();

            // Calculate percentiles per day
            let mut p50_data: Vec<f32> = Vec::new();
            let mut p90_data: Vec<f32> = Vec::new();
            let mut p100_data: Vec<f32> = Vec::new();
            let mut x_labels: Vec<String> = Vec::new();

            for date in &all_dates {
                let mut times = lead_times_by_day.get(date).cloned().unwrap_or_default();
                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let p50 = calculate_percentile(&times, 50.0) as f32;
                let p90 = calculate_percentile(&times, 90.0) as f32;
                let p100 = calculate_percentile(&times, 100.0) as f32;
                p50_data.push(p50);
                p90_data.push(p90);
                p100_data.push(p100);
                x_labels.push(date.format("%a").to_string());
            }

            let mut chart = BarChart::new_with_theme(
                vec![
                    Series::new("p50".to_string(), p50_data),
                    Series::new("p90".to_string(), p90_data),
                    Series::new("p100".to_string(), p100_data),
                ],
                x_labels,
                THEME_DARK,
            );

            chart.width = 700.0;
            chart.height = 400.0;
            chart.background_color = NACRE_BG;
            chart.series_colors = vec![NACRE_BLUE, NACRE_GREEN, NACRE_ORANGE];
            chart.y_axis_configs[0].axis_formatter = Some("{c}h".to_string());
            chart.series_list[0].label_show = true;
            chart.series_list[1].label_show = true;
            chart.series_list[2].label_show = true;

            chart.svg().unwrap_or_default()
        } else {
            String::new()
        }
    };

    // Generate Cycle Time Percentiles Chart (p50, p90, p100 over time) using charts-rs
    let cycle_time_distribution_svg = {
        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(7);

        // Group closed issues by close date and calculate cycle times
        let mut cycle_times_by_day: HashMap<chrono::NaiveDate, Vec<f64>> = HashMap::new();
        for issue in &all_issues {
            if let Some(closed_at) = issue.closed_at {
                let close_date = closed_at.date_naive();
                if close_date >= start_dt.date_naive() {
                    if let Some(started_at) = started_times.get(&issue.id) {
                        let duration_mins = (closed_at - *started_at).num_minutes() as f64;
                        cycle_times_by_day
                            .entry(close_date)
                            .or_default()
                            .push(duration_mins);
                    }
                }
            }
        }

        if !cycle_times_by_day.is_empty() {
            // Collect and sort dates
            let mut all_dates: Vec<chrono::NaiveDate> = cycle_times_by_day.keys().cloned().collect();
            all_dates.sort();

            // Calculate percentiles per day
            let mut p50_data: Vec<f32> = Vec::new();
            let mut p90_data: Vec<f32> = Vec::new();
            let mut p100_data: Vec<f32> = Vec::new();
            let mut x_labels: Vec<String> = Vec::new();

            for date in &all_dates {
                let mut times = cycle_times_by_day.get(date).cloned().unwrap_or_default();
                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let p50 = calculate_percentile(&times, 50.0) as f32;
                let p90 = calculate_percentile(&times, 90.0) as f32;
                let p100 = calculate_percentile(&times, 100.0) as f32;
                p50_data.push(p50);
                p90_data.push(p90);
                p100_data.push(p100);
                x_labels.push(date.format("%a").to_string());
            }

            let mut chart = BarChart::new_with_theme(
                vec![
                    Series::new("p50".to_string(), p50_data),
                    Series::new("p90".to_string(), p90_data),
                    Series::new("p100".to_string(), p100_data),
                ],
                x_labels,
                THEME_DARK,
            );

            chart.width = 700.0;
            chart.height = 400.0;
            chart.background_color = NACRE_BG;
            chart.series_colors = vec![NACRE_BLUE, NACRE_GREEN, NACRE_ORANGE];
            chart.y_axis_configs[0].axis_formatter = Some("{c}m".to_string());
            chart.series_list[0].label_show = true;
            chart.series_list[1].label_show = true;
            chart.series_list[2].label_show = true;

            chart.svg().unwrap_or_default()
        } else {
            String::new()
        }
    };

    // Generate Throughput Chart (Date-based) using charts-rs
    let throughput_distribution_svg = {
        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(7);

        let mut throughput_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();
        // Fill in all days with 0 first
        let mut curr = start_dt.date_naive();
        while curr <= now_dt.date_naive() {
            throughput_by_day.insert(curr, 0);
            curr = curr.succ_opt().unwrap();
        }

        for issue in &all_issues {
            if let Some(closed_at) = issue.closed_at {
                let resolved_date = closed_at.date_naive();
                if resolved_date >= start_dt.date_naive() {
                    *throughput_by_day.entry(resolved_date).or_insert(0) += 1;
                }
            }
        }

        // Collect and sort dates
        let mut all_dates: Vec<chrono::NaiveDate> = throughput_by_day.keys().cloned().collect();
        all_dates.sort();

        let mut throughput_data: Vec<f32> = Vec::new();
        let mut x_labels: Vec<String> = Vec::new();
        for date in &all_dates {
            let count = *throughput_by_day.get(date).unwrap_or(&0);
            throughput_data.push(count as f32);
            x_labels.push(date.format("%a").to_string());
        }

        let mut chart = BarChart::new_with_theme(
            vec![Series::new("Closed".to_string(), throughput_data)],
            x_labels,
            THEME_DARK,
        );

        chart.width = 700.0;
        chart.height = 400.0;
        chart.background_color = NACRE_BG;
        chart.series_colors = vec![NACRE_BLUE];
        chart.series_list[0].label_show = true;

        chart.svg().unwrap_or_default()
    };

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
        tickets_chart_svg,
        lead_time_chart_svg,
        cycle_time_distribution_svg,
        throughput_distribution_svg,
        p50_lead_time_hours,
        p90_lead_time_hours,
        p100_lead_time_hours,
        p50_cycle_time_mins,
        p90_cycle_time_mins,
        p100_cycle_time_mins,
    }
}
