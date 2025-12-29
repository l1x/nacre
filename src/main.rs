use argh::FromArgs;
use nacre::{AppState, create_app};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        .with(tracing_subscriber::fmt::layer().with_timer(
            tracing_subscriber::fmt::time::UtcTime::new(kiters::timestamp::get_utc_formatter()),
        ))
        .init();

    let args: Args = argh::from_env();
    let state = Arc::new(AppState::new());

    let app = create_app(state);

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
