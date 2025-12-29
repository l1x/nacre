mod beads;
mod error;
mod handlers;
mod templates;

pub use error::{AppError, AppResult};

use argh::FromArgs;
use axum::Router;
use axum::routing::{get, post};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tower_http::trace::TraceLayer;
use tracing::Span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

pub struct AppState {
    pub client: beads::Client,
    pub project_name: String,
    pub app_version: String,
}

// Arc wrapper for shared state
pub type SharedAppState = Arc<AppState>;

impl AppState {
    fn new() -> Self {
        let project_name = std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "Nacre".to_string());

        Self {
            client: beads::Client::new(),
            project_name,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

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

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nacre=info,tower_http=info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(tracing_subscriber::fmt::time::UtcTime::new(
                    kiters::timestamp::get_utc_formatter(),
                )),
        )
        .init();

    let args: Args = argh::from_env();
    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/", get(handlers::landing))
        .route("/tasks", get(handlers::tasks_list))
        .route("/tasks/new", get(handlers::new_task_form))
        .route("/tasks/:id", get(handlers::task_detail))
        .route("/tasks/:id/edit", get(handlers::edit_task))
        .route("/board", get(handlers::board))
        .route("/graph", get(handlers::graph))
        .route("/metrics", get(handlers::metrics_handler))
        .route("/prds", get(handlers::prds_list))
        .route("/prds/:filename", get(handlers::prd_view))
        .route("/api/issues", get(handlers::list_tasks))
        .route("/api/issues/:id", post(handlers::update_task))
        .route("/api/issues", post(handlers::create_task))
        .route("/health", get(handlers::health_check))
        .route("/style.css", get(handlers::serve_css))
        .route("/app.js", get(handlers::serve_js))
        .route("/favicon.ico", get(handlers::serve_favicon))
        .route("/favicon.svg", get(handlers::serve_favicon))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    static REQUEST_ID: AtomicU64 = AtomicU64::new(1);
                    let request_id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
                    tracing::info_span!(
                        "request",
                        id = request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_request(|request: &axum::http::Request<_>, _span: &Span| {
                    tracing::info!("-> {} {}", request.method(), request.uri());
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &Span| {
                        tracing::info!(
                            "<- {} latency={}µs",
                            response.status().as_u16(),
                            latency.as_micros()
                        );
                    },
                ),
        );

    let addr_str = format!("{}:{}", args.host, args.port);
    let addr: SocketAddr = addr_str.parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;
    let url = format!("http://{}", actual_addr);

    tracing::info!("{}", url);

    if args.open
        && let Err(e) = open::that(&url)
    {
        tracing::error!("Failed to open browser: {}", e);
    }

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_state() -> SharedAppState {
        Arc::new(AppState::new())
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = Router::new().route("/health", get(handlers::health_check));

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
        let app = Router::new()
            .route("/", get(handlers::landing))
            .with_state(test_state());

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
    async fn test_tasks() {
        let app = Router::new()
            .route("/tasks", get(handlers::tasks_list))
            .with_state(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks")
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
    async fn test_board() {
        let app = Router::new()
            .route("/board", get(handlers::board))
            .with_state(test_state());

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
        let app = Router::new()
            .route("/graph", get(handlers::graph))
            .with_state(test_state());

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
    async fn test_task_detail() {
        let app = Router::new()
            .route("/tasks/:id", get(handlers::task_detail))
            .with_state(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks/nacre-90b.3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_edit_task() {
        let app = Router::new()
            .route("/tasks/:id/edit", get(handlers::edit_task))
            .with_state(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks/nacre-90b.3/edit")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let app = Router::new()
            .route("/api/issues", get(handlers::list_tasks))
            .with_state(test_state());

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
