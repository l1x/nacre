mod beads;

use axum::{
    routing::get,
    Json, Router,
};
use argh::FromArgs;
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
    #[argh(option, short = 's', default = "String::from(\"frontend/dist\")")]
    static_dir: String,
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
        .nest_service("/", ServeDir::new(&args.static_dir))
        .route("/api/issues", get(list_issues))
        .route("/health", get(health_check))
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
    async fn test_static_files() {
        let app = Router::new().nest_service("/", ServeDir::new("frontend/dist"));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(body.starts_with(b"<h1>Nacre</h1>"));
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