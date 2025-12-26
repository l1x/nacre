use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use plotters::prelude::*;
use pulldown_cmark::{Parser, html};
use std::collections::HashMap;

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
            started_times
                .entry(act.issue_id.clone())
                .or_insert(act.timestamp);
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

    LandingTemplate {
        project_name: state.project_name.clone(),
        page_title: String::new(),
        active_nav: "dashboard",
        stats,
        epics,
        blocked,
        in_progress,
    }
}

pub async fn index(State(state): State<crate::AppState>) -> IndexTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();
    let nodes = build_issue_tree(&all_issues);

    IndexTemplate {
        project_name: state.project_name.clone(),
        page_title: "Issues".to_string(),
        active_nav: "issues",
        nodes,
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
        page_title: "Epics".to_string(),
        active_nav: "epics",
        epics,
    }
}

pub async fn epic_detail(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
) -> crate::AppResult<EpicDetailTemplate> {
    let all_issues = state.client.list_issues()?;

    // Find the epic
    let epic = all_issues
        .iter()
        .find(|i| i.id == id && i.issue_type == beads::IssueType::Epic)
        .ok_or_else(|| crate::AppError::NotFound(format!("Epic {}", id)))?;

    Ok(EpicDetailTemplate {
        project_name: state.project_name.clone(),
        page_title: id.clone(),
        active_nav: "epics",
        epic: EpicWithProgress::from_epic(epic, &all_issues, true),
    })
}

pub async fn board(State(state): State<crate::AppState>) -> BoardTemplate {
    let all_issues = state.client.list_issues().unwrap_or_default();

    let columns = vec![
        BoardColumn {
            name: "Open".to_string(),
            status: "open".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Open)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "In Progress".to_string(),
            status: "in_progress".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::InProgress)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Blocked".to_string(),
            status: "blocked".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Blocked)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Deferred".to_string(),
            status: "deferred".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Deferred)
                .cloned()
                .collect(),
        },
        BoardColumn {
            name: "Closed".to_string(),
            status: "closed".to_string(),
            issues: all_issues
                .iter()
                .filter(|i| i.status == beads::Status::Closed)
                .cloned()
                .collect(),
        },
    ];

    BoardTemplate {
        project_name: state.project_name.clone(),
        page_title: "Board".to_string(),
        active_nav: "board",
        columns,
    }
}

/// Build a hierarchical tree of issues for display
fn build_issue_tree(all_issues: &[beads::Issue]) -> Vec<TreeNode> {
    // Build parent-child relationships
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut parent_map: HashMap<String, String> = HashMap::new();

    for issue in all_issues {
        // Check explicit parent-child dependency
        for dep in &issue.dependencies {
            if dep.dep_type == beads::DependencyType::ParentChild {
                children_map
                    .entry(dep.depends_on_id.clone())
                    .or_default()
                    .push(issue.id.clone());
                parent_map.insert(issue.id.clone(), dep.depends_on_id.clone());
            }
        }

        // Check dot-notation for implicit parent-child (e.g., nacre-3hd.1 -> nacre-3hd)
        if !parent_map.contains_key(&issue.id) {
            if let Some(dot_pos) = issue.id.rfind('.') {
                let potential_parent = &issue.id[..dot_pos];
                if all_issues.iter().any(|i| i.id == potential_parent) {
                    children_map
                        .entry(potential_parent.to_string())
                        .or_default()
                        .push(issue.id.clone());
                    parent_map.insert(issue.id.clone(), potential_parent.to_string());
                }
            }
        }
    }

    // Count blocking dependencies (non-parent-child) for each issue
    let mut blocked_by_count: HashMap<String, usize> = HashMap::new();
    for issue in all_issues {
        let count = issue
            .dependencies
            .iter()
            .filter(|d| d.dep_type != beads::DependencyType::ParentChild)
            .count();
        blocked_by_count.insert(issue.id.clone(), count);
    }

    // Build issue lookup
    let issue_map: HashMap<String, &beads::Issue> =
        all_issues.iter().map(|i| (i.id.clone(), i)).collect();

    // Recursive function to build tree nodes
    fn build_tree(
        issue_id: &str,
        issue_map: &HashMap<String, &beads::Issue>,
        children_map: &HashMap<String, Vec<String>>,
        blocked_by_count: &HashMap<String, usize>,
        depth: usize,
        nodes: &mut Vec<TreeNode>,
    ) {
        let Some(issue) = issue_map.get(issue_id) else {
            return;
        };

        let children_ids = children_map.get(issue_id);
        let has_children = children_ids.map(|c| !c.is_empty()).unwrap_or(false);

        // Determine parent_id for this node
        let parent_id = if depth > 0 {
            if let Some(dot_pos) = issue.id.rfind('.') {
                Some(issue.id[..dot_pos].to_string())
            } else {
                None
            }
        } else {
            None
        };

        nodes.push(TreeNode {
            id: issue.id.clone(),
            title: issue.title.clone(),
            status: issue.status.as_str().to_string(),
            issue_type: issue.issue_type.as_css_class().to_string(),
            priority: issue.priority.unwrap_or(2),
            blocked_by_count: blocked_by_count.get(&issue.id).copied().unwrap_or(0),
            has_children,
            depth,
            parent_id,
        });

        // Recursively add children
        if let Some(children) = children_ids {
            let mut sorted_children: Vec<_> = children
                .iter()
                .filter_map(|id| issue_map.get(id).map(|i| (id, *i)))
                .collect();
            sorted_children.sort_by(|a, b| {
                a.1.status
                    .sort_order()
                    .cmp(&b.1.status.sort_order())
                    .then_with(|| a.0.cmp(b.0))
            });

            for (child_id, _) in sorted_children {
                build_tree(
                    child_id,
                    issue_map,
                    children_map,
                    blocked_by_count,
                    depth + 1,
                    nodes,
                );
            }
        }
    }

    // Find top-level nodes (no parent)
    let mut top_level: Vec<&beads::Issue> = all_issues
        .iter()
        .filter(|i| !parent_map.contains_key(&i.id))
        .collect();

    // Sort top-level: epics first, then by status, then by id
    top_level.sort_by(|a, b| {
        let a_is_epic = a.issue_type == beads::IssueType::Epic;
        let b_is_epic = b.issue_type == beads::IssueType::Epic;
        b_is_epic
            .cmp(&a_is_epic)
            .then_with(|| a.status.sort_order().cmp(&b.status.sort_order()))
            .then_with(|| a.id.cmp(&b.id))
    });

    // Build flat tree
    let mut nodes = Vec::new();
    for issue in top_level {
        build_tree(
            &issue.id,
            &issue_map,
            &children_map,
            &blocked_by_count,
            0,
            &mut nodes,
        );
    }

    nodes
}

pub async fn graph(State(state): State<crate::AppState>) -> GraphTemplate {
    GraphTemplate {
        project_name: state.project_name.clone(),
        page_title: "Graph".to_string(),
        active_nav: "graph",
    }
}

pub async fn issue_detail(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
) -> crate::AppResult<IssueDetailTemplate> {
    let issue = state.client.get_issue(&id)?;
    Ok(IssueDetailTemplate {
        project_name: state.project_name.clone(),
        page_title: id,
        active_nav: "",
        issue,
    })
}

pub async fn edit_issue(
    State(state): State<crate::AppState>,
    Path(id): Path<String>,
) -> crate::AppResult<EditIssueTemplate> {
    let issue = state.client.get_issue(&id)?;
    Ok(EditIssueTemplate {
        project_name: state.project_name.clone(),
        page_title: format!("Edit {}", id),
        active_nav: "",
        issue,
    })
}

pub async fn new_issue_form(State(state): State<crate::AppState>) -> NewIssueTemplate {
    NewIssueTemplate {
        project_name: state.project_name.clone(),
        page_title: "New Issue".to_string(),
        active_nav: "",
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
        page_title: "PRDs".to_string(),
        active_nav: "prds",
        files,
    }
}

pub async fn prd_view(
    State(state): State<crate::AppState>,
    Path(filename): Path<String>,
) -> crate::AppResult<PrdViewTemplate> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(crate::AppError::BadRequest("Invalid filename".to_string()));
    }

    let path = format!("docs/prds/{}", filename);
    let markdown_input =
        std::fs::read_to_string(&path).map_err(|_| crate::AppError::NotFound(filename.clone()))?;

    let parser = Parser::new(&markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Ok(PrdViewTemplate {
        project_name: state.project_name.clone(),
        page_title: filename.clone(),
        active_nav: "prds",
        filename,
        content: html_output,
    })
}

pub async fn list_issues(
    State(state): State<crate::AppState>,
) -> crate::AppResult<Json<Vec<beads::Issue>>> {
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
            .caption(
                "Tickets Activity (Last 30 Days)",
                ("sans-serif", 20).into_font(),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                start_dt.date_naive()..now_dt.date_naive(),
                0..10usize, // Initial Y scale, will be updated if needed
            )
            .unwrap();

        // Adjust Y scale based on data
        let max_v = created_by_day
            .values()
            .chain(resolved_by_day.values())
            .max()
            .copied()
            .unwrap_or(5)
            .max(5);
        chart
            .configure_mesh()
            .x_labels(10)
            .y_labels(5)
            .draw()
            .unwrap();

        // Re-build with correct Y scale
        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Tickets Activity (Last 30 Days)",
                ("sans-serif", 20).into_font(),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(start_dt.date_naive()..now_dt.date_naive(), 0..max_v + 1)
            .unwrap();

        chart
            .configure_mesh()
            .x_label_formatter(&|d| d.format("%m-%d").to_string())
            .draw()
            .unwrap();

        let mut created_data: Vec<(chrono::NaiveDate, usize)> =
            created_by_day.into_iter().collect();
        created_data.sort_by_key(|(d, _)| *d);

        let mut resolved_data: Vec<(chrono::NaiveDate, usize)> =
            resolved_by_day.into_iter().collect();
        resolved_data.sort_by_key(|(d, _)| *d);

        chart
            .draw_series(LineSeries::new(created_data, BLUE))
            .unwrap()
            .label("Created")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        chart
            .draw_series(LineSeries::new(resolved_data, GREEN))
            .unwrap()
            .label("Resolved")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();
    }

    MetricsTemplate {
        project_name: state.project_name.clone(),
        page_title: "Metrics".to_string(),
        active_nav: "metrics",
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
