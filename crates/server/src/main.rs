use std::net::SocketAddr;
use std::time::Duration;

use clap::Parser;
use cubacle_server::{router, AppState, Config};
use tracing_subscriber::EnvFilter;

/// Stateless HTTP backend for the cubacle solvers.
#[derive(Parser)]
#[command(name = "cubacle-server", version, about)]
struct Args {
    /// Address to listen on (local-only by default)
    #[arg(long, env = "CUBACLE_BIND", default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    /// Solves that may run at once (default: number of CPUs)
    #[arg(long, env = "CUBACLE_MAX_CONCURRENCY")]
    max_concurrency: Option<usize>,
    /// Upper bound and default for one solve, in milliseconds
    #[arg(long, env = "CUBACLE_REQUEST_TIMEOUT_MS", default_value_t = 30_000)]
    request_timeout_ms: u64,
    /// Largest accepted request body in bytes
    #[arg(long, env = "CUBACLE_MAX_BODY_BYTES", default_value_t = 16 * 1024)]
    max_body_bytes: usize,
    /// Log filter, e.g. info or cubacle_server=debug
    #[arg(long, env = "CUBACLE_LOG", default_value = "info")]
    log: String,
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(&args.log).unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let mut config = Config {
        request_timeout: Duration::from_millis(args.request_timeout_ms),
        max_body_bytes: args.max_body_bytes,
        ..Config::default()
    };
    if let Some(n) = args.max_concurrency {
        config.max_concurrency = n.max(1);
    }
    tracing::info!(concurrency = config.max_concurrency, "preparing engines");
    // Table construction is CPU-bound; keep it off the async threads.
    let state = match tokio::task::spawn_blocking(move || AppState::new(config)).await {
        Ok(state) => state,
        Err(error) => {
            tracing::error!("engine preparation failed: {error}");
            return std::process::ExitCode::FAILURE;
        }
    };

    let listener = match tokio::net::TcpListener::bind(args.bind).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!("cannot bind {}: {error}", args.bind);
            return std::process::ExitCode::FAILURE;
        }
    };
    tracing::info!("listening on {}", args.bind);
    if let Err(error) = axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown())
        .await
    {
        tracing::error!("server error: {error}");
        return std::process::ExitCode::FAILURE;
    }
    tracing::info!("shut down");
    std::process::ExitCode::SUCCESS
}

/// Resolves on SIGINT or SIGTERM; in-flight requests then finish.
async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = term => {} }
}
