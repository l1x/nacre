mod beads;
mod error;
mod handlers;
mod templates;

pub use error::{AppError, AppResult};

use argh::FromArgs;
use axum::Router;
use axum::routing::{get, post};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub type AppState = beads::Client;

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
    let client = beads::Client::new();

    let app = Router::new()
        .route("/", get(handlers::landing))
        .route("/issues", get(handlers::index))
        .route("/epics", get(handlers::epics))
        .route("/epics/:id", get(handlers::epic_detail))
        .route("/board", get(handlers::board))
        .route("/graph", get(handlers::graph))
        .route("/metrics", get(handlers::metrics_handler))
        .route("/issues/new", get(handlers::new_issue_form))
        .route("/issues/:id", get(handlers::issue_detail))
        .route("/prds", get(handlers::prds_list))
        .route("/prds/:filename", get(handlers::prd_view))
        .route("/api/issues", get(handlers::list_issues))
        .route("/api/issues/:id", post(handlers::update_issue_handler))
        .route("/api/issues", post(handlers::create_issue_handler))
        .route("/health", get(handlers::health_check))
        .route("/style.css", get(handlers::serve_css))
        .route("/app.js", get(handlers::serve_js))
        .with_state(client)
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_client() -> beads::Client {
        beads::Client::new()
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
            .with_state(test_client());

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
        let app = Router::new()
            .route("/issues", get(handlers::index))
            .with_state(test_client());

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
        let app = Router::new()
            .route("/epics", get(handlers::epics))
            .with_state(test_client());

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
        let app = Router::new()
            .route("/board", get(handlers::board))
            .with_state(test_client());

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
            .with_state(test_client());

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
        let app = Router::new()
            .route("/issues/:id", get(handlers::issue_detail))
            .with_state(test_client());

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
        let app = Router::new()
            .route("/api/issues", get(handlers::list_issues))
            .with_state(test_client());

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
