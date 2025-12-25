mod beads;

use argh::FromArgs;
use askama::Template;
use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
use pulldown_cmark::{Parser, html};
use std::collections::HashSet;
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(FromArgs, Debug)]
/// Nacre: A local-first web interface for Beads.
struct Args {
    /// host to bind to
    #[argh(option, default = "String::from(\"127.0.0.1\")")]
    host: String,

    /// port to listen on
    #[argh(option, short = 'p', default = "3000")]
    port: u16,

    /// open the browser automatically
    #[argh(switch, short = 'o')]
    open: bool,

    /// directory to serve static files from
    #[argh(option, short = 's', default = "String::from(\"frontend/public\")")]
    static_dir: String,
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

struct EpicWithProgress {
    issue: beads::Issue,
    total: usize,
    closed: usize,
    percent: f64,
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
        .route("/", get(index))
        .route("/epics", get(epics))
        .route("/board", get(board))
        .route("/issues/:id", get(issue_detail))
        .route("/prds", get(prds_list))
        .route("/prds/:filename", get(prd_view))
        .route("/api/issues", get(list_issues))
        .route("/api/issues/:id", post(update_issue_handler))
        .route("/health", get(health_check))
        .fallback_service(ServeDir::new(&args.static_dir))
        .layer(TraceLayer::new_for_http());

    let addr_str = format!("{}:{}", args.host, args.port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid host or port");

    tracing::info!("listening on {}", addr);

    if args.open {
        let url = format!("http://{}", addr);
        if let Err(e) = open::that(&url) {
            tracing::error!("Failed to open browser: {}", e);
        }
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
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
        let children: Vec<&beads::Issue> = all_issues
            .iter()
            .filter(|i| {
                i.dependencies.iter().any(|d| d.depends_on_id == epic.id)
                    || i.id.starts_with(&prefix)
            })
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
        });
    }

    // Sort epics by most recently updated first
    epics.sort_by(|a, b| b.issue.updated_at.cmp(&a.issue.updated_at));

    EpicsTemplate { epics }
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
    async fn test_index() {
        let app = Router::new().route("/", get(index));

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
