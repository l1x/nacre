mod beads;

use argh::FromArgs;
use askama::Template;
use axum::{
    Json, Router,
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use pulldown_cmark::{Parser, html};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use plotters::prelude::*;

mod filters {
    pub fn format_hours(hours: &f64) -> askama::Result<String> {
        Ok(format!("{:.1}h", hours))
    }
    pub fn format_decimal(val: &f64) -> askama::Result<String> {
        Ok(format!("{:.2}", val))
    }
    pub fn format_date(date: &chrono::DateTime<chrono::FixedOffset>) -> askama::Result<String> {
        Ok(date.format("%Y-%m-%d %H:%M").to_string())
    }
}

// Embed static assets at compile time
const STYLE_CSS: &str = include_str!("../frontend/public/style.css");
const APP_JS: &str = include_str!("../frontend/public/app.js");

#[derive(FromArgs, Debug)]
/// Nacre: A local-first web interface for Beads.
struct Args {
    /// host to bind to
    #[argh(option, default = "String::from(\"127.0.0.1\")")]
    host: String,

    /// port to listen on (0 for random available port)
    #[argh(option, short = 'p', default = "0")]
    port: u16,

    /// open the browser automatically
    #[argh(switch, short = 'o')]
    open: bool,
}

#[derive(Template)]
#[template(path = "landing.html")]
struct LandingTemplate {
    stats: ProjectStats,
    epics: Vec<EpicWithProgress>,
    blocked: Vec<beads::Issue>,
    in_progress: Vec<beads::Issue>,
}

struct ProjectStats {
    total: usize,
    open: usize,
    in_progress: usize,
    blocked: usize,
    closed: usize,
    avg_lead_time_hours: f64,
    avg_cycle_time_hours: f64,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    groups: Vec<IssueGroup>,
}

struct IssueGroup {
    epic_title: String,
    issues: Vec<beads::Issue>,
}

#[derive(Template)]
#[template(path = "epics.html")]
struct EpicsTemplate {
    epics: Vec<EpicWithProgress>,
}

#[derive(Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    columns: Vec<BoardColumn>,
}

#[derive(Template)]
#[template(path = "issue.html")]
struct IssueDetailTemplate {
    issue: beads::Issue,
}

#[derive(Template)]
#[template(path = "new_issue.html")]
struct NewIssueTemplate {}

#[derive(Template)]
#[template(path = "prds.html")]
struct PrdsListTemplate {
    files: Vec<String>,
}

#[derive(Template)]
#[template(path = "prd.html")]
struct PrdViewTemplate {
    filename: String,
    content: String,
}

#[derive(Template)]
#[template(path = "epic_detail.html")]
struct EpicDetailTemplate {
    epic: EpicWithProgress,
}

#[derive(Template)]
#[template(path = "graph.html")]
struct GraphTemplate {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    width: i32,
    height: i32,
}

#[derive(Template)]
#[template(path = "metrics.html")]
struct MetricsTemplate {
    avg_lead_time_hours: f64,
    avg_cycle_time_hours: f64,
    throughput_per_day: f64,
    closed_last_7_days: usize,
    wip_count: usize,
    blocked_count: usize,
    tickets_chart_svg: String,
}

struct GraphNode {
    id: String,
    title: String,
    title_short: String,
    status: String,
    issue_type: String,
    priority: u8,
    parent_id: Option<String>,
    is_epic: bool,
    x: i32,
    y: i32,
}

struct GraphEdge {
    source_id: String,
    target_id: String,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

struct EpicWithProgress {
    issue: beads::Issue,
    total: usize,
    closed: usize,
    percent: f64,
    children: Vec<beads::Issue>,
}

struct BoardColumn {
    name: String,
    issues: Vec<beads::Issue>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nacre=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args: Args = argh::from_env();

    let app = Router::new()
        .route("/", get(landing))
        .route("/issues", get(index))
        .route("/epics", get(epics))
        .route("/epics/:id", get(epic_detail))
        .route("/board", get(board))
        .route("/graph", get(graph))
        .route("/metrics", get(metrics_handler))
        .route("/issues/new", get(new_issue_form))
        .route("/issues/:id", get(issue_detail))
        .route("/prds", get(prds_list))
        .route("/prds/:filename", get(prd_view))
        .route("/api/issues", get(list_issues))
        .route("/api/issues/:id", post(update_issue_handler))
        .route("/api/issues", post(create_issue_handler))
        .route("/health", get(health_check))
        .route("/style.css", get(serve_css))
        .route("/app.js", get(serve_js))
        .layer(TraceLayer::new_for_http());

    let addr_str = format!("{}:{}", args.host, args.port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid host or port");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let actual_addr = listener.local_addr().unwrap();
    let url = format!("http://{}", actual_addr);

    tracing::info!("{}", url);

    if args.open && let Err(e) = open::that(&url) {
        tracing::error!("Failed to open browser: {}", e);
    }

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn serve_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

async fn serve_js() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/javascript")], APP_JS)
}

async fn landing() -> LandingTemplate {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();
    let activities = client.get_activity().unwrap_or_default();
    let summary = client.get_status_summary().unwrap_or_default();

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
    let epic_issues: Vec<&beads::Issue> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic && i.status != beads::Status::Closed)
        .collect();

    let mut epics: Vec<EpicWithProgress> = Vec::new();
    for epic in epic_issues {
        let prefix = format!("{}.", epic.id);
        let children: Vec<beads::Issue> = all_issues
            .iter()
            .filter(|i| {
                i.dependencies.iter().any(|d| d.depends_on_id == epic.id)
                    || i.id.starts_with(&prefix)
            })
            .cloned()
            .collect();

        let total = children.len();
        let closed = children
            .iter()
            .filter(|i| i.status == beads::Status::Closed)
            .count();
        let percent = if total > 0 {
            (closed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        epics.push(EpicWithProgress {
            issue: epic.clone(),
            total,
            closed,
            percent,
            children: Vec::new(), // Not needed for landing
        });
    }

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
        stats,
        epics,
        blocked,
        in_progress,
    }
}

async fn index() -> IndexTemplate {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();

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

    IndexTemplate { groups }
}

async fn epics() -> EpicsTemplate {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();

    let mut epics: Vec<EpicWithProgress> = Vec::new();

    // Filter epics
    let epic_issues: Vec<&beads::Issue> = all_issues
        .iter()
        .filter(|i| i.issue_type == beads::IssueType::Epic)
        .collect();

    for epic in epic_issues {
        let prefix = format!("{}.", epic.id);
        let mut children: Vec<beads::Issue> = all_issues
            .iter()
            .filter(|i| {
                i.dependencies.iter().any(|d| d.depends_on_id == epic.id)
                    || i.id.starts_with(&prefix)
            })
            .cloned()
            .collect();

        // Sort children by status priority
        children.sort_by_key(|i| i.status.sort_order());

        let total = children.len();
        let closed = children
            .iter()
            .filter(|i| i.status == beads::Status::Closed)
            .count();
        let percent = if total > 0 {
            (closed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        epics.push(EpicWithProgress {
            issue: epic.clone(),
            total,
            closed,
            percent,
            children,
        });
    }

    // Sort epics by most recently updated first
    epics.sort_by(|a, b| b.issue.updated_at.cmp(&a.issue.updated_at));

    EpicsTemplate { epics }
}

async fn epic_detail(Path(id): Path<String>) -> Result<EpicDetailTemplate, StatusCode> {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();

    // Find the epic
    let epic_issue = all_issues
        .iter()
        .find(|i| i.id == id && i.issue_type == beads::IssueType::Epic)
        .cloned();

    match epic_issue {
        Some(epic) => {
            let prefix = format!("{}.", epic.id);
            let mut children: Vec<beads::Issue> = all_issues
                .iter()
                .filter(|i| {
                    i.dependencies.iter().any(|d| d.depends_on_id == epic.id)
                        || i.id.starts_with(&prefix)
                })
                .cloned()
                .collect();

            // Sort children by status priority
            children.sort_by_key(|i| i.status.sort_order());

            let total = children.len();
            let closed = children
                .iter()
                .filter(|i| i.status == beads::Status::Closed)
                .count();
            let percent = if total > 0 {
                (closed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            Ok(EpicDetailTemplate {
                epic: EpicWithProgress {
                    issue: epic,
                    total,
                    closed,
                    percent,
                    children,
                },
            })
        }
        None => {
            tracing::error!("Epic not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn board() -> BoardTemplate {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();

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

    BoardTemplate { columns }
}

async fn graph() -> GraphTemplate {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();

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
        nodes: graph_nodes,
        edges: graph_edges,
        width: svg_width,
        height: svg_height,
    }
}

async fn issue_detail(Path(id): Path<String>) -> Result<IssueDetailTemplate, StatusCode> {
    let client = beads::Client::new();
    match client.get_issue(&id) {
        Ok(issue) => Ok(IssueDetailTemplate { issue }),
        Err(e) => {
            tracing::error!("Failed to get issue {}: {}", id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn new_issue_form() -> NewIssueTemplate {
    NewIssueTemplate {}
}

async fn prds_list() -> PrdsListTemplate {
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
    PrdsListTemplate { files }
}

async fn prd_view(Path(filename): Path<String>) -> Result<PrdViewTemplate, StatusCode> {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let path = format!("docs/prds/{}", filename);
    match std::fs::read_to_string(&path) {
        Ok(markdown_input) => {
            let parser = Parser::new(&markdown_input);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            Ok(PrdViewTemplate {
                filename,
                content: html_output,
            })
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn list_issues() -> Json<Vec<beads::Issue>> {
    let client = beads::Client::new();
    match client.list_issues() {
        Ok(issues) => Json(issues),
        Err(e) => {
            tracing::error!("Failed to list issues: {}", e);
            Json(vec![])
        }
    }
}

async fn metrics_handler() -> MetricsTemplate {
    let client = beads::Client::new();
    let all_issues = client.list_issues().unwrap_or_default();
    let activities = client.get_activity().unwrap_or_default();
    let summary = client.get_status_summary().unwrap_or_default();

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
        avg_lead_time_hours,
        avg_cycle_time_hours,
        throughput_per_day,
        closed_last_7_days,
        wip_count,
        blocked_count,
        tickets_chart_svg,
    }
}

async fn update_issue_handler(
    Path(id): Path<String>,
    Json(update): Json<beads::IssueUpdate>,
) -> StatusCode {
    let client = beads::Client::new();
    match client.update_issue(&id, update) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to update issue: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn create_issue_handler(
    Json(create): Json<beads::IssueCreate>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let client = beads::Client::new();
    match client.create_issue(create) {
        Ok(id) => Ok(Json(serde_json::json!({ "id": id }))),
        Err(e) => {
            tracing::error!("Failed to create issue: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health", get(health_check));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn test_landing() {
        let app = Router::new().route("/", get(landing));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(body.starts_with(b"<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn test_index() {
        let app = Router::new().route("/issues", get(index));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/issues")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(body.starts_with(b"<!DOCTYPE html>"));
    }

    #[tokio::test]
    async fn test_epics() {
        let app = Router::new().route("/epics", get(epics));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/epics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_board() {
        let app = Router::new().route("/board", get(board));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/board")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_graph() {
        let app = Router::new().route("/graph", get(graph));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/graph")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_issue_detail() {
        let app = Router::new().route("/issues/:id", get(issue_detail));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/issues/nacre-p1b")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_issues() {
        let app = Router::new().route("/api/issues", get(list_issues));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/issues")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
