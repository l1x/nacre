use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::beads;
use crate::handlers;

/// Format latency in human-readable units
fn format_latency(duration: std::time::Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1000 {
        format!("{}µs", micros)
    } else if micros < 1_000_000 {
        format!("{}ms", micros / 1000)
    } else {
        format!("{:.1}s", micros as f64 / 1_000_000.0)
    }
}

pub struct AppState {
    pub client: beads::Client,
    pub project_name: String,
    pub app_version: String,
}

pub type SharedAppState = Arc<AppState>;

impl AppState {
    pub fn new() -> Self {
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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_app(state: SharedAppState) -> Router {
    Router::new()
        .route("/", get(handlers::landing))
        .route("/tasks", get(handlers::tasks_list))
        .route("/tasks/new", get(handlers::new_task_form))
        .route("/tasks/:id", get(handlers::task_detail))
        .route("/tasks/:id/edit", get(handlers::edit_task))
        .route("/board", get(handlers::board))
        .route("/graph", get(handlers::graph))
        .route("/metrics", get(handlers::metrics_handler))
        .route("/palette", get(handlers::palette))
        .route("/prds", get(handlers::prds_list))
        .route("/prds/:filename", get(handlers::prd_view))
        .route("/api/issues", get(handlers::list_tasks))
        .route("/api/issues/:id", post(handlers::update_task))
        .route("/api/issues", post(handlers::create_task))
        .route("/api/graph", get(handlers::graph_data))
        .route("/health", get(handlers::health_check))
        .route("/style.css", get(handlers::serve_css))
        .route("/autumnus.dark.css", get(handlers::serve_autumnus_dark))
        .route("/autumnus.light.css", get(handlers::serve_autumnus_light))
        .route("/app.js", get(handlers::serve_js))
        .route("/favicon.ico", get(handlers::serve_favicon))
        .route("/favicon.svg", get(handlers::serve_favicon))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    static REQUEST_ID: AtomicU64 = AtomicU64::new(1);
                    let request_id_num = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
                    let generator = block_id::BlockId::new(
                        block_id::Alphabet::alphanumeric(),
                        1234,
                        5,
                    );
                    let request_id = generator
                        .encode_string(request_id_num)
                        .unwrap_or_else(|| request_id_num.to_string());
                    tracing::info_span!(
                        "request",
                        id = %request_id,
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
                            "<- {} latency={}",
                            response.status().as_u16(),
                            format_latency(latency)
                        );
                    },
                ),
        )
        .layer(CompressionLayer::new())
}
