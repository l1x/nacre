use axum::extract::State;
use plotters::prelude::*;
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

    // Generate Chart
    let mut tickets_chart_svg = String::new();
    {
        // Theme colors matching others
        let color_created = RGBColor(79, 129, 189); // Blue
        let color_resolved = RGBColor(155, 187, 89); // Green
        
        let bg_color = RGBColor(35, 31, 29);
        let text_color = RGBColor(154, 149, 144);
        let grid_color = RGBColor(34, 32, 32);

        let root = SVGBackend::with_string(&mut tickets_chart_svg, (800, 400)).into_drawing_area();
        root.fill(&bg_color).unwrap();

        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(30);

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

        let mut day_data: Vec<(String, usize, usize)> = Vec::new(); // (label, created, resolved)
        let mut max_v = 0;

        for date in &all_dates {
            let created = *created_by_day.get(date).unwrap_or(&0);
            let resolved = *resolved_by_day.get(date).unwrap_or(&0);
            day_data.push((date.format("%m-%d").to_string(), created, resolved));
            if created + resolved > max_v {
                max_v = created + resolved;
            }
        }

        // Ensure Y axis has some height
        if max_v == 0 { max_v = 5; }

        let num_days = day_data.len();
        let bar_padding = 0.10;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Tickets Activity (Last 30 Days)",
                ("sans-serif", 20).into_font().color(&text_color),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0f64..(num_days as f64), 0usize..(max_v + 1))
            .unwrap();

        chart
            .configure_mesh()
            .bold_line_style(grid_color)
            .light_line_style(grid_color.mix(0.5))
            .x_labels(num_days)
            .x_label_formatter(&|x| {
                let idx = x.round() as usize;
                if idx < day_data.len() && (*x - idx as f64).abs() < 0.3 {
                    day_data[idx].0.clone()
                } else {
                    String::new()
                }
            })
            .y_labels(5)
            .axis_style(text_color)
            .label_style(("sans-serif", 12).into_font().color(&text_color))
            .draw()
            .unwrap();

        // Draw stacked bars
        for (idx, (_, created, resolved)) in day_data.iter().enumerate() {
            let x_left = idx as f64 + bar_padding;
            let x_right = (idx + 1) as f64 - bar_padding;
            
            // Created bar (bottom)
            if *created > 0 {
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(x_left, 0), (x_right, *created)],
                        color_created.filled(),
                    )))
                    .unwrap();
            }

            // Resolved bar (stacked on top of created)
            if *resolved > 0 {
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(x_left, *created), (x_right, *created + *resolved)],
                        color_resolved.filled(),
                    )))
                    .unwrap();
            }
        }

        // Legend
        let legend_items = [
            (color_created, "Created"),
            (color_resolved, "Resolved"),
        ];
        let legend_start_x = 300i32;
        let legend_spacing = 100i32;

        for (i, (color, label)) in legend_items.iter().enumerate() {
            let x = legend_start_x + (i as i32) * legend_spacing;
            root.draw(&Rectangle::new(
                [(x, 370), (x + 20, 385)],
                color.filled(),
            ))
            .unwrap();
            root.draw(&Text::new(
                *label,
                (x + 25, 373),
                ("sans-serif", 13).into_font().color(&text_color),
            ))
            .unwrap();
        }
    }

    // Generate Lead Time Percentiles Chart (p50, p90, p100 over time)
    let mut lead_time_chart_svg = String::new();
    {
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
            // Calculate percentiles for each day
            fn percentile(sorted: &[f64], p: f64) -> f64 {
                if sorted.is_empty() {
                    return 0.0;
                }
                let idx = ((sorted.len() as f64 - 1.0) * p / 100.0).round() as usize;
                sorted[idx.min(sorted.len() - 1)]
            }

            // Format hours for display
            fn format_hours(h: f64) -> String {
                if h >= 24.0 {
                    format!("{:.1}d", h / 24.0)
                } else {
                    format!("{:.1}h", h)
                }
            }

            // Collect and sort dates
            let mut all_dates: Vec<chrono::NaiveDate> = lead_times_by_day.keys().cloned().collect();
            all_dates.sort();

            // Calculate percentiles per day
            let mut day_data: Vec<(String, f64, f64, f64)> = Vec::new(); // (label, p50, p90, p100)
            for date in &all_dates {
                let mut times = lead_times_by_day.get(date).cloned().unwrap_or_default();
                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let p50 = percentile(&times, 50.0);
                let p90 = percentile(&times, 90.0);
                let p100 = percentile(&times, 100.0);
                day_data.push((date.format("%a").to_string(), p50, p90, p100));
            }

            let max_hours = day_data
                .iter()
                .map(|(_, _, _, p100)| *p100)
                .fold(1.0_f64, f64::max);

            // Theme colors matching the "good example" but adapted for dark mode
            let color_p50 = RGBColor(79, 129, 189); // Direct (Blue)
            let color_p90 = RGBColor(155, 187, 89); // Mail Ad (Green)
            let color_p100 = RGBColor(247, 150, 70); // Video Ad (Orange)
            
            // Match CSS --bg-card: #231f1d
            let bg_color = RGBColor(35, 31, 29);
            // Match CSS --text-secondary: #9a9590
            let text_color = RGBColor(154, 149, 144);
            // Match CSS --border-subtle: #222020
            let grid_color = RGBColor(34, 32, 32);

            let root =
                SVGBackend::with_string(&mut lead_time_chart_svg, (700, 400)).into_drawing_area();
            root.fill(&bg_color).unwrap();

            let num_days = day_data.len();
            let bar_padding = 0.10; // Gap between bars (10% on each side = 80% bar width)

            let mut chart = ChartBuilder::on(&root)
                .x_label_area_size(40)
                .y_label_area_size(50)
                .margin(20)
                .margin_bottom(60) // Extra space for legend
                .build_cartesian_2d(0f64..(num_days as f64), 0f64..(max_hours * 1.1))
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .bold_line_style(grid_color)
                .light_line_style(grid_color.mix(0.5))
                .y_desc("Lead Time")
                .y_label_formatter(&|v| format_hours(*v))
                .x_labels(num_days)
                .x_label_formatter(&|x| {
                    let idx = x.round() as usize;
                    if idx < day_data.len() && (*x - idx as f64).abs() < 0.3 {
                        // Parse date string (MM-DD) back to date object to get weekday
                        // Or just modify day_data generation to include weekday
                        // For now, we'll try to parse the MM-DD if possible, or just use what we have.
                        // Actually, day_data only has MM-DD string. 
                        // Let's modify the day_data generation above to include the weekday.
                        // But since I can't easily modify the code above in this replace block without replacing huge chunk...
                        // I will rely on the fact that I can't easily change day_data type here.
                        // Wait, I can't change the day_data type in this scope?
                        // I am replacing the drawing part. The day_data generation is ABOVE this block.
                        // So I must stick to using the string in day_data.0
                        // However, I can try to append weekday if I modify the generation logic.
                        // But the `replace` tool requires EXACT match. 
                        // I am replacing the block starting from `// Theme colors` to the end of the legend loop.
                        
                        // I will change the day_data generation in a separate call if needed.
                        // For now, let's just stick to the MM-DD label but use the new colors/style.
                        // Or I can parse the MM-DD relative to current year?
                        // It's safer to just leave MM-DD for now or do a separate refactor.
                        // BUT the requirement was "Mon", "Tue".
                        
                        // Let's look at day_data generation. It is OUTSIDE the block I am replacing?
                        // No, the `replace` block I selected starts at `// Theme colors`.
                        // The day_data generation is ABOVE it.
                        // "let mut day_data: Vec<(String, f64, f64, f64)> = Vec::new();"
                        // I need to include day_data generation in the replacement to change the label.
                        
                        day_data[idx].0.clone()
                    } else {
                        String::new()
                    }
                })
                .axis_desc_style(("sans-serif", 14).into_font().color(&text_color))
                .label_style(("sans-serif", 12).into_font().color(&text_color))
                .axis_style(text_color)
                .draw()
                .unwrap();

            // Draw stacked bars with gaps between days
            for (idx, (_, p50, p90, p100)) in day_data.iter().enumerate() {
                let x_left = idx as f64 + bar_padding;
                let x_right = (idx + 1) as f64 - bar_padding;
                let x_center = idx as f64 + 0.5;
                let label_font = ("sans-serif", 12).into_font().color(&WHITE);

                // p50 segment (bottom): 0 to p50 - BLUE
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(x_left, 0.0), (x_right, *p50)],
                        color_p50.filled(),
                    )))
                    .unwrap();

                // p50 label (always show if segment is tall enough)
                if *p50 > max_hours * 0.06 {
                    chart
                        .draw_series(std::iter::once(Text::new(
                            format_hours(*p50),
                            (x_center, *p50 / 2.0),
                            label_font.clone(),
                        )))
                        .unwrap();
                }

                // p90 segment (middle): p50 to p90 - GREEN
                if *p90 > *p50 {
                    chart
                        .draw_series(std::iter::once(Rectangle::new(
                            [(x_left, *p50), (x_right, *p90)],
                            color_p90.filled(),
                        )))
                        .unwrap();

                    // p90 label
                    if (*p90 - *p50) > max_hours * 0.06 {
                        chart
                            .draw_series(std::iter::once(Text::new(
                                format_hours(*p90),
                                (x_center, *p50 + (*p90 - *p50) / 2.0),
                                label_font.clone(),
                            )))
                            .unwrap();
                    }
                }

                // p100 segment (top): p90 to p100 - ORANGE
                if *p100 > *p90 {
                    chart
                        .draw_series(std::iter::once(Rectangle::new(
                            [(x_left, *p90), (x_right, *p100)],
                            color_p100.filled(),
                        )))
                        .unwrap();

                    // p100 label
                    if (*p100 - *p90) > max_hours * 0.06 {
                        chart
                            .draw_series(std::iter::once(Text::new(
                                format_hours(*p100),
                                (x_center, *p90 + (*p100 - *p90) / 2.0),
                                label_font.clone(),
                            )))
                            .unwrap();
                    }
                }
            }

            // Draw legend at bottom center
            let legend_items = [
                (color_p50, "p50"),
                (color_p90, "p90"),
                (color_p100, "p100"),
            ];
            let legend_start_x = 250i32;
            let legend_spacing = 80i32;

            for (i, (color, label)) in legend_items.iter().enumerate() {
                let x = legend_start_x + (i as i32) * legend_spacing;
                // Draw colored rectangle
                root.draw(&Rectangle::new(
                    [(x, 370), (x + 20, 385)],
                    color.filled(),
                ))
                .unwrap();
                // Draw label text
                root.draw(&Text::new(
                    *label,
                    (x + 25, 373),
                    ("sans-serif", 13).into_font().color(&text_color),
                ))
                .unwrap();
            }
        }
    }

    // Generate Cycle Time Percentiles Chart (p50, p90, p100 over time)
    let mut cycle_time_distribution_svg = String::new();
    {
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
            let mut day_data: Vec<(String, f64, f64, f64)> = Vec::new(); // (label, p50, p90, p100)
            for date in &all_dates {
                let mut times = cycle_times_by_day.get(date).cloned().unwrap_or_default();
                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let p50 = calculate_percentile(&times, 50.0);
                let p90 = calculate_percentile(&times, 90.0);
                let p100 = calculate_percentile(&times, 100.0);
                day_data.push((date.format("%a").to_string(), p50, p90, p100));
            }

            let max_mins = day_data
                .iter()
                .map(|(_, _, _, p100)| *p100)
                .fold(1.0_f64, f64::max);

            // Theme colors
            let color_p50 = RGBColor(79, 129, 189); // Blue
            let color_p90 = RGBColor(155, 187, 89); // Green
            let color_p100 = RGBColor(247, 150, 70); // Orange
            
            let bg_color = RGBColor(35, 31, 29);
            let text_color = RGBColor(154, 149, 144);
            let grid_color = RGBColor(34, 32, 32);

            let root =
                SVGBackend::with_string(&mut cycle_time_distribution_svg, (700, 400)).into_drawing_area();
            root.fill(&bg_color).unwrap();

            let num_days = day_data.len();
            let bar_padding = 0.10;

            let mut chart = ChartBuilder::on(&root)
                .x_label_area_size(40)
                .y_label_area_size(50)
                .margin(20)
                .margin_bottom(60)
                .build_cartesian_2d(0f64..(num_days as f64), 0f64..(max_mins * 1.1))
                .unwrap();

            chart
                .configure_mesh()
                .disable_x_mesh()
                .bold_line_style(grid_color)
                .light_line_style(grid_color.mix(0.5))
                .y_desc("Cycle Time (min)")
                .y_label_formatter(&|v| format!("{:.0}m", v))
                .x_labels(num_days)
                .x_label_formatter(&|x| {
                    let idx = x.round() as usize;
                    if idx < day_data.len() && (*x - idx as f64).abs() < 0.3 {
                        day_data[idx].0.clone()
                    } else {
                        String::new()
                    }
                })
                .axis_desc_style(("sans-serif", 14).into_font().color(&text_color))
                .label_style(("sans-serif", 12).into_font().color(&text_color))
                .axis_style(text_color)
                .draw()
                .unwrap();

            // Draw stacked bars
            for (idx, (_, p50, p90, p100)) in day_data.iter().enumerate() {
                let x_left = idx as f64 + bar_padding;
                let x_right = (idx + 1) as f64 - bar_padding;
                let x_center = idx as f64 + 0.5;
                let label_font = ("sans-serif", 12).into_font().color(&WHITE);

                // p50 segment
                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [(x_left, 0.0), (x_right, *p50)],
                        color_p50.filled(),
                    )))
                    .unwrap();

                if *p50 > max_mins * 0.06 {
                    chart
                        .draw_series(std::iter::once(Text::new(
                            format!("{:.0}", p50),
                            (x_center, *p50 / 2.0),
                            label_font.clone(),
                        )))
                        .unwrap();
                }

                // p90 segment
                if *p90 > *p50 {
                    chart
                        .draw_series(std::iter::once(Rectangle::new(
                            [(x_left, *p50), (x_right, *p90)],
                            color_p90.filled(),
                        )))
                        .unwrap();

                    if (*p90 - *p50) > max_mins * 0.06 {
                        chart
                            .draw_series(std::iter::once(Text::new(
                                format!("{:.0}", p90),
                                (x_center, *p50 + (*p90 - *p50) / 2.0),
                                label_font.clone(),
                            )))
                            .unwrap();
                    }
                }

                // p100 segment
                if *p100 > *p90 {
                    chart
                        .draw_series(std::iter::once(Rectangle::new(
                            [(x_left, *p90), (x_right, *p100)],
                            color_p100.filled(),
                        )))
                        .unwrap();

                    if (*p100 - *p90) > max_mins * 0.06 {
                        chart
                            .draw_series(std::iter::once(Text::new(
                                format!("{:.0}", p100),
                                (x_center, *p90 + (*p100 - *p90) / 2.0),
                                label_font.clone(),
                            )))
                            .unwrap();
                    }
                }
            }

            // Draw legend
            let legend_items = [
                (color_p50, "p50"),
                (color_p90, "p90"),
                (color_p100, "p100"),
            ];
            let legend_start_x = 250i32;
            let legend_spacing = 80i32;

            for (i, (color, label)) in legend_items.iter().enumerate() {
                let x = legend_start_x + (i as i32) * legend_spacing;
                root.draw(&Rectangle::new(
                    [(x, 370), (x + 20, 385)],
                    color.filled(),
                ))
                .unwrap();
                root.draw(&Text::new(
                    *label,
                    (x + 25, 373),
                    ("sans-serif", 13).into_font().color(&text_color),
                ))
                .unwrap();
            }
        }
    }

    // Generate Throughput Chart (Date-based)
    let mut throughput_distribution_svg = String::new();
    {
        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(30);
        
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

        let mut day_data: Vec<(String, usize)> = Vec::new();
        for date in &all_dates {
            let count = *throughput_by_day.get(date).unwrap_or(&0);
            day_data.push((date.format("%m-%d").to_string(), count));
        }

        let max_val = *throughput_by_day.values().max().unwrap_or(&0);

        // Theme colors
        let bg_color = RGBColor(35, 31, 29);
        let text_color = RGBColor(154, 149, 144);
        let grid_color = RGBColor(34, 32, 32);
        let color_throughput = RGBColor(155, 187, 89); // Green matching p90/Resolved

        let root = SVGBackend::with_string(&mut throughput_distribution_svg, (700, 400)).into_drawing_area();
        root.fill(&bg_color).unwrap();

        let num_days = day_data.len();
        let bar_padding = 0.10;

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .margin(20)
            .margin_bottom(60)
            .build_cartesian_2d(0f64..(num_days as f64), 0usize..(max_val + 1))
            .unwrap();

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(grid_color)
            .light_line_style(grid_color.mix(0.5))
            .y_desc("Issues Closed")
            .x_labels(num_days)
            .x_label_formatter(&|x| {
                let idx = x.round() as usize;
                if idx < day_data.len() && (*x - idx as f64).abs() < 0.3 {
                    day_data[idx].0.clone()
                } else {
                    String::new()
                }
            })
            .axis_desc_style(("sans-serif", 14).into_font().color(&text_color))
            .label_style(("sans-serif", 12).into_font().color(&text_color))
            .axis_style(text_color)
            .draw()
            .unwrap();

        // Draw bars
        for (idx, (_, count)) in day_data.iter().enumerate() {
            let x_left = idx as f64 + bar_padding;
            let x_right = (idx + 1) as f64 - bar_padding;
            
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x_left, 0), (x_right, *count)],
                    color_throughput.filled(),
                )))
                .unwrap();
        }
    }

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
