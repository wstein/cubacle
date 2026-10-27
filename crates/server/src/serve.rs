//! The accept loop with header timeout and bounded drain.

use std::time::Duration;

use axum::Router;

use crate::state::*;

/// Serves `app` on `listener` until `shutdown` resolves, then drains.
///
/// `axum::serve` has no header-read timeout, so one half-open connection
/// could keep a graceful shutdown waiting forever. This loop uses hyper's
/// HTTP/1 server directly (hyper and hyper-util are already axum's
/// dependencies, so nothing new is compiled) to get a header-read timeout,
/// and bounds the drain by `request_timeout + shutdown_grace`; connections
/// still open after that are dropped.
///
/// Returns `true` if everything drained in time.
pub async fn serve(
    listener: tokio::net::TcpListener,
    app: Router,
    config: &Config,
    shutdown: impl std::future::Future<Output = ()>,
) -> bool {
    use hyper_util::rt::{TokioIo, TokioTimer};
    use hyper_util::server::graceful::GracefulShutdown;
    use hyper_util::service::TowerToHyperService;

    let graceful = GracefulShutdown::new();
    let header_timeout = config.header_read_timeout;
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let stream = match accepted {
                    Ok((stream, _)) => stream,
                    Err(error) => {
                        tracing::warn!("accept failed: {error}");
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        continue;
                    }
                };
                let service = TowerToHyperService::new(app.clone());
                let mut http = hyper::server::conn::http1::Builder::new();
                http.timer(TokioTimer::new()).header_read_timeout(header_timeout);
                let connection = http.serve_connection(TokioIo::new(stream), service);
                let connection = graceful.watch(connection);
                tokio::spawn(async move {
                    let _ = connection.await;
                });
            }
            () = &mut shutdown => break,
        }
    }
    drop(listener);
    let deadline = config.request_timeout + config.shutdown_grace;
    tokio::time::timeout(deadline, graceful.shutdown())
        .await
        .is_ok()
}
