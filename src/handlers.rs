use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use plotters::prelude::*;
use pulldown_cmark::{Parser, html};
use std::collections::{HashMap, HashSet};

use crate::beads;
use crate::templates::*;

// Embed static assets at compile time
const STYLE_CSS: &str = include_str!("../frontend/public/style.css");
const APP_JS: &str = include_str!("../frontend/public/app.js");
const FAVICON_SVG: &str = include_str!("../frontend/public/favicon.svg");

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn serve_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

pub async fn serve_js() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/javascript")], APP_JS)
}

pub async fn serve_favicon() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}

pub async fn landing(State(state): State<crate::AppState>) -> LandingTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();
    let activities = state.client.get_activity().unwrap_or_default();
    let summary = state.client.get_status_summary().unwrap_or_default();

    let avg_lead_time_hours = summary["summary"]["average_lead_time_hours"]
        .as_f64()
        .unwrap_or(0.0);

    // Calculate Cycle Time
    let mut started_times: HashMap<String, chrono::DateTime<chrono::FixedOffset>> = HashMap::new();
    for act in &activities {
        if act.new_status == Some(beads::Status::InProgress) {
            started_times.entry(act.issue_id.clone()).or_insert(act.timestamp);
        }
    }

    let mut cycle_times = Vec::new();
    for issue in &all_issues {
        if let Some(closed_at) = issue.closed_at
            && let Some(started_at) = started_times.get(&issue.id)
        {
            let duration = closed_at - *started_at;
            cycle_times.push(duration.num_minutes() as f64 / 60.0);
        }
    }

    let avg_cycle_time_hours = if !cycle_times.is_empty() {
        cycle_times.iter().sum::<f64>() / cycle_times.len() as f64
    } else {
        0.0
    };

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
        avg_lead_time_hours,
        avg_cycle_time_hours,
    };

    // Get epics with progress
    let mut epics: Vec<EpicWithProgress> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic && i.status != beads::Status::Closed)
        .map(|epic| EpicWithProgress::from_epic(epic, &all_issues, false))
        .collect();

    // Sort epics by percent complete (least complete first to highlight work needed)
    epics.sort_by(|a, b| a.percent.partial_cmp(&b.percent).unwrap_or(std::cmp::Ordering::Equal));

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

    LandingTemplate {
        project_name: state.project_name.clone(),
        stats,
        epics,
        blocked,
        in_progress,
    }
}

pub async fn index(State(state): State<crate::AppState>) -> IndexTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();

    let mut epics: Vec<beads::Issue> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic)
        .cloned()
        .collect();

    // Sort epics by most recently updated first
    epics.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    let mut groups: Vec<IssueGroup> = Vec::new();

    for epic in &epics {
        let prefix = format!("{}.", epic.id);
        let mut children: Vec<beads::Issue> = all_issues
            .iter()
            .filter(|i| {
                i.dependencies.iter().any(|d| d.depends_on_id == epic.id)
                    || i.id.starts_with(&prefix)
            })
            .cloned()
            .collect();

        // Sort by status priority
        children.sort_by_key(|i| i.status.sort_order());

        if !children.is_empty() {
            groups.push(IssueGroup {
                epic_title: epic.title.clone(),
                issues: children,
            });
        }
    }

    // Un-grouped issues (excluding epics themselves and issues already in groups)
    let grouped_ids: HashSet<String> = groups
        .iter()
        .flat_map(|g| g.issues.iter().map(|i| i.id.clone()))
        .collect();

    let mut un_grouped: Vec<beads::Issue> = all_issues
        .iter()
        .filter(|i| i.issue_type != beads::IssueType::Epic && !grouped_ids.contains(&i.id))
        .cloned()
        .collect();

    // Sort by status priority
    un_grouped.sort_by_key(|i| i.status.sort_order());

    if !un_grouped.is_empty() {
        groups.push(IssueGroup {
            epic_title: "No Epic".to_string(),
            issues: un_grouped,
        });
    }

    IndexTemplate {
        project_name: state.project_name.clone(),
        groups,
    }
}

pub async fn epics(State(state): State<crate::AppState>) -> EpicsTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();

    let mut epics: Vec<EpicWithProgress> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic)
        .map(|epic| EpicWithProgress::from_epic(epic, &all_issues, true))
        .collect();

    // Sort epics by most recently updated first
    epics.sort_by(|a, b| b.issue.updated_at.cmp(&a.issue.updated_at));

    EpicsTemplate {
        project_name: state.project_name.clone(),
        epics,
    }
}

pub async fn epic_detail(State(state): State<crate::AppState>, Path(id): Path<String>) -> crate::AppResult<EpicDetailTemplate> {
    let all_issues = state.client.list_issues()?;

    // Find the epic
    let epic = all_issues
        .iter()
        .find(|i| i.id == id && i.issue_type == beads::IssueType::Epic)
        .ok_or_else(|| crate::AppError::NotFound(format!("Epic {}", id)))?;

    Ok(EpicDetailTemplate {
        project_name: state.project_name.clone(),
        epic: EpicWithProgress::from_epic(epic, &all_issues, true),
    })
}

pub async fn board(State(state): State<crate::AppState>) -> BoardTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();

    let columns = vec![
        BoardColumn {
            name: "Blocked".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Blocked)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Ready".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Open)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "In Progress".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::InProgress)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Closed".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Closed)
                .cloned()
                .collect(),
        },
    ];

    BoardTemplate {
        project_name: state.project_name.clone(),
        columns,
    }
}

pub async fn graph(State(state): State<crate::AppState>) -> GraphTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();

    // Build dependency graph
    let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    // Initialize all nodes
    for issue in &all_issues {
        dependents.entry(issue.id.clone()).or_default();
        in_degree.entry(issue.id.clone()).or_insert(0);
    }

    // Build edges from dependencies
    for issue in &all_issues {
        for dep in &issue.dependencies {
            // issue depends on dep.depends_on_id, so dep.depends_on_id -> issue
            dependents
                .entry(dep.depends_on_id.clone())
                .or_default()
                .push(issue.id.clone());
            *in_degree.entry(issue.id.clone()).or_insert(0) += 1;
        }
    }

    // Topological sort with levels (BFS)
    let mut levels: HashMap<String, usize> = HashMap::new();
    let mut queue: Vec<String> = Vec::new();

    // Start with nodes that have no dependencies
    for issue in &all_issues {
        if in_degree.get(&issue.id).copied().unwrap_or(0) == 0 {
            queue.push(issue.id.clone());
            levels.insert(issue.id.clone(), 0);
        }
    }

    let mut max_level = 0;
    while !queue.is_empty() {
        let current = queue.remove(0);
        let current_level = levels.get(&current).copied().unwrap_or(0);

        if let Some(deps) = dependents.get(&current) {
            for dep_id in deps {
                let new_level = current_level + 1;
                let existing_level = levels.entry(dep_id.clone()).or_insert(0);
                if new_level > *existing_level {
                    *existing_level = new_level;
                }
                max_level = max_level.max(new_level);

                let deg = in_degree.get_mut(dep_id).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dep_id.clone());
                }
            }
        }
    }

    // Group nodes by level
    let mut nodes_by_level: Vec<Vec<&beads::Issue>> = vec![Vec::new(); max_level + 1];
    for issue in &all_issues {
        let level = levels.get(&issue.id).copied().unwrap_or(0);
        if level < nodes_by_level.len() {
            nodes_by_level[level].push(issue);
        }
    }

    // Calculate positions
    let node_width = 180;
    let node_height = 80;
    let level_gap = 100;
    let node_gap = 40;

    let mut max_width = 0;
    for level_nodes in &nodes_by_level {
        let level_width = level_nodes.len() as i32 * (node_width + node_gap);
        max_width = max_width.max(level_width);
    }

    let svg_width = max_width.max(600) + 100;
    let svg_height = ((max_level + 1) as i32 * (node_height + level_gap)) + 100;

    let mut node_positions: HashMap<String, (i32, i32)> = HashMap::new();
    let mut graph_nodes = Vec::new();

    for (level, level_nodes) in nodes_by_level.iter().enumerate() {
        let total_width = level_nodes.len() as i32 * (node_width + node_gap) - node_gap;
        let start_x = (svg_width - total_width) / 2 + node_width / 2;
        let y = 50 + level as i32 * (node_height + level_gap) + node_height / 2;

        for (i, issue) in level_nodes.iter().enumerate() {
            let x = start_x + i as i32 * (node_width + node_gap);
            node_positions.insert(issue.id.clone(), (x, y));

            // Truncate title
            let title_short = if issue.title.len() > 20 {
                format!("{}...", &issue.title[..17])
            } else {
                issue.title.clone()
            };

            graph_nodes.push(GraphNode {
                id: issue.id.clone(),
                title: issue.title.clone(),
                title_short,
                status: issue.status.as_str().to_string(),
                issue_type: issue.issue_type.as_css_class().to_string(),
                priority: issue.priority.unwrap_or(2),
                parent_id: issue.dependencies.iter()
                    .find(|d| d.dep_type == "parent-child")
                    .map(|d| d.depends_on_id.clone()),
                is_epic: issue.issue_type == beads::IssueType::Epic,
                x,
                y,
            });
        }
    }

    // Build edges
    let mut graph_edges = Vec::new();
    for issue in &all_issues {
        if let Some(&(x2, y2)) = node_positions.get(&issue.id) {
            for dep in &issue.dependencies {
                if let Some(&(x1, y1)) = node_positions.get(&dep.depends_on_id) {
                    // Edge from dependency to dependent (top to bottom)
                    graph_edges.push(GraphEdge {
                        source_id: dep.depends_on_id.clone(),
                        target_id: issue.id.clone(),
                        x1,
                        y1: y1 + 30, // Bottom of source node
                        x2,
                        y2: y2 - 30, // Top of target node
                    });
                }
            }
        }
    }

    GraphTemplate {
        project_name: state.project_name.clone(),
        nodes: graph_nodes,
        edges: graph_edges,
        width: svg_width,
        height: svg_height,
    }
}

pub async fn issue_detail(State(state): State<crate::AppState>, Path(id): Path<String>) -> crate::AppResult<IssueDetailTemplate> {
    let issue = state.client.get_issue(&id)?;
    Ok(IssueDetailTemplate {
        project_name: state.project_name.clone(),
        issue,
    })
}

pub async fn new_issue_form(State(state): State<crate::AppState>) -> NewIssueTemplate {
    NewIssueTemplate {
        project_name: state.project_name.clone(),
    }
}

pub async fn prds_list(State(state): State<crate::AppState>) -> PrdsListTemplate {
    let mut files_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("docs/prds") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string()
                && name.ends_with(".md")
            {
                let modified = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files_with_time.push((name, modified));
            }
        }
    }
    // Sort by most recently modified first
    files_with_time.sort_by(|a, b| b.1.cmp(&a.1));
    let files: Vec<String> = files_with_time.into_iter().map(|(name, _)| name).collect();
    PrdsListTemplate {
        project_name: state.project_name.clone(),
        files,
    }
}

pub async fn prd_view(State(state): State<crate::AppState>, Path(filename): Path<String>) -> crate::AppResult<PrdViewTemplate> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(crate::AppError::BadRequest("Invalid filename".to_string()));
    }

    let path = format!("docs/prds/{}", filename);
    let markdown_input = std::fs::read_to_string(&path)
        .map_err(|_| crate::AppError::NotFound(filename.clone()))?;

    let parser = Parser::new(&markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(PrdViewTemplate {
        project_name: state.project_name.clone(),
        filename,
        content: html_output,
    })
}

pub async fn list_issues(State(state): State<crate::AppState>) -> crate::AppResult<Json<Vec<beads::Issue>>> {
    let issues = state.client.list_issues()?;
    Ok(Json(issues))
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
            started_times.entry(act.issue_id.clone()).or_insert(act.timestamp);
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
                cycle_times.push(duration.num_minutes() as f64 / 60.0);
            }
        }
    }

    let avg_cycle_time_hours = if !cycle_times.is_empty() {
        cycle_times.iter().sum::<f64>() / cycle_times.len() as f64
    } else {
        0.0
    };

    let throughput_per_day = closed_last_7_days as f64 / 7.0;

    let wip_count = all_issues
        .iter()
        .filter(|i| i.status == beads::Status::InProgress)
        .count();
    let blocked_count = all_issues
        .iter()
        .filter(|i| i.status == beads::Status::Blocked)
        .count();

    // Generate Chart
    let mut tickets_chart_svg = String::new();
    {
        let root = SVGBackend::with_string(&mut tickets_chart_svg, (800, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let now_dt = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let start_dt = now_dt - chrono::Duration::days(30);

        let mut created_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();
        let mut resolved_by_day: HashMap<chrono::NaiveDate, usize> = HashMap::new();

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

        let mut chart = ChartBuilder::on(&root)
            .caption("Tickets Activity (Last 30 Days)", ("sans-serif", 20).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                start_dt.date_naive()..now_dt.date_naive(),
                0..10usize, // Initial Y scale, will be updated if needed
            ).unwrap();

        // Adjust Y scale based on data
        let max_v = created_by_day.values().chain(resolved_by_day.values()).max().copied().unwrap_or(5).max(5);
        chart.configure_mesh()
            .x_labels(10)
            .y_labels(5)
            .draw().unwrap();

        // Re-build with correct Y scale
        let mut chart = ChartBuilder::on(&root)
            .caption("Tickets Activity (Last 30 Days)", ("sans-serif", 20).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                start_dt.date_naive()..now_dt.date_naive(),
                0..max_v + 1,
            ).unwrap();

        chart.configure_mesh()
            .x_label_formatter(&|d| d.format("%m-%d").to_string())
            .draw().unwrap();

        let mut created_data: Vec<(chrono::NaiveDate, usize)> = created_by_day.into_iter().collect();
        created_data.sort_by_key(|(d, _)| *d);

        let mut resolved_data: Vec<(chrono::NaiveDate, usize)> = resolved_by_day.into_iter().collect();
        resolved_data.sort_by_key(|(d, _)| *d);

        chart.draw_series(
            LineSeries::new(created_data, BLUE),
        ).unwrap()
        .label("Created")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        chart.draw_series(
            LineSeries::new(resolved_data, GREEN),
        ).unwrap()
        .label("Resolved")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

        chart.configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw().unwrap();
    }

    MetricsTemplate {
        project_name: state.project_name.clone(),
        avg_lead_time_hours,
        avg_cycle_time_hours,
        throughput_per_day,
        closed_last_7_days,
        wip_count,
        blocked_count,
        tickets_chart_svg,
    }
}

pub async fn update_issue_handler(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
    Json(update): Json<beads::IssueUpdate>,
) -> crate::AppResult<StatusCode> {
    state.client.update_issue(&id, update)?;
    Ok(StatusCode::OK)
}

pub async fn create_issue_handler(
    State(state): State<crate::AppState>,
    Json(create): Json<beads::IssueCreate>,
) -> crate::AppResult<Json<serde_json::Value>> {
    let id = state.client.create_issue(create)?;
    Ok(Json(serde_json::json!({ "id": id })))
}
