mod beads;

use argh::FromArgs;
use askama::Template;
use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
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
    #[argh(option, short = 'p', default = "4000")]
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
    issues: Vec<beads::Issue>,
}

#[derive(Template)]
#[template(path = "epics.html")]
struct EpicsTemplate {
    epics: Vec<EpicWithProgress>,
}

struct EpicWithProgress {
    issue: beads::Issue,
    total: usize,
    closed: usize,
    percent: f64,
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
    let issues = client.list_issues().unwrap_or_default();
    IndexTemplate { issues }
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
        // Find children: starts with epic.id + "."
        let prefix = format!("{}.", epic.id);
        let children: Vec<&beads::Issue> = all_issues
            .iter()
            .filter(|i| i.id.starts_with(&prefix))
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

    EpicsTemplate { epics }
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
}
