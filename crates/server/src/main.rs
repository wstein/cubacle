use std::net::SocketAddr;
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use cubacle_server::{router, serve, AppState, Config, MAX_CONCURRENCY, MAX_TIMEOUT_MS};
use tracing_subscriber::EnvFilter;

/// Stateless HTTP backend for the cubacle solvers.
#[derive(Parser)]
#[command(name = "cubacle-server", version, about)]
struct Args {
    /// Address to listen on (local-only by default)
    #[arg(long, env = "CUBACLE_BIND", default_value = "127.0.0.1:8080")]
    bind: SocketAddr,
    /// Solves that may run at once, 1..=256 (default: number of CPUs, at most 256)
    #[arg(long, env = "CUBACLE_MAX_CONCURRENCY",
          value_parser = clap::value_parser!(u32).range(1..=MAX_CONCURRENCY as i64))]
    max_concurrency: Option<u32>,
    /// Upper bound and default for one solve, in milliseconds, 1..=600000
    #[arg(long, env = "CUBACLE_REQUEST_TIMEOUT_MS", default_value_t = 30_000,
          value_parser = clap::value_parser!(u64).range(1..=MAX_TIMEOUT_MS))]
    request_timeout_ms: u64,
    /// Largest accepted request body in bytes
    #[arg(long, env = "CUBACLE_MAX_BODY_BYTES", default_value_t = 16 * 1024)]
    max_body_bytes: usize,
    /// Log filter, e.g. info or cubacle_server=debug
    #[arg(long, env = "CUBACLE_LOG", default_value = "info")]
    log: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(&args.log).unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let mut config = Config {
        request_timeout: Duration::from_millis(args.request_timeout_ms),
        max_body_bytes: args.max_body_bytes,
        ..Config::default()
    };
    config.max_concurrency = match args.max_concurrency {
        Some(n) => n as usize,
        None => config.max_concurrency.min(MAX_CONCURRENCY),
    };
    let grace = config.shutdown_grace;

    // Blocking threads: one per permit plus headroom for engine preparation,
    // so a permit holder never waits for a thread.
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(config.max_concurrency + 4)
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!("cannot start the runtime: {error}");
            return ExitCode::FAILURE;
        }
    };

    let code = runtime.block_on(run(args.bind, config));
    // Orphaned blocking solves are abandoned after the grace period; the
    // process exits right after, so "shut down" is logged only once the
    // runtime really stopped.
    runtime.shutdown_timeout(grace);
    tracing::info!("shut down");
    code
}

async fn run(bind: SocketAddr, config: Config) -> ExitCode {
    tracing::info!(concurrency = config.max_concurrency, "preparing engines");
    // Table construction is CPU-bound; keep it off the async threads.
    let prepared = config.clone();
    let state = match tokio::task::spawn_blocking(move || AppState::new(prepared)).await {
        Ok(state) => state,
        Err(error) => {
            tracing::error!("engine preparation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let listener = match tokio::net::TcpListener::bind(bind).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!("cannot bind {bind}: {error}");
            return ExitCode::FAILURE;
        }
    };
    match listener.local_addr() {
        Ok(addr) => tracing::info!("listening on {addr}"),
        Err(_) => tracing::info!("listening on {bind}"),
    }
    let drained = serve(listener, router(state), &config, shutdown()).await;
    if !drained {
        tracing::warn!("connections still open after the shutdown deadline were dropped");
    }
    ExitCode::SUCCESS
}

/// Resolves on the first SIGINT or SIGTERM and arms an immediate exit for the
/// next one (tokio replaced the default handler, so without this a second
/// Ctrl-C would be ignored).
async fn shutdown() {
    wait_signal().await;
    tracing::info!("shutdown requested; a second signal exits immediately");
    tokio::spawn(async {
        wait_signal().await;
        std::process::exit(130);
    });
}

async fn wait_signal() {
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
    tokio::select! { () = ctrl_c => {}, () = term => {} }
}
