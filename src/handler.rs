use axum::http::{StatusCode, Uri};
use stream_cancel::{Trigger, Tripwire};
use tokio::signal;
use tracing::error;
use tracing::info;

#[cfg(not(tarpaulin_include))]
///handle the shutdown signal
pub async fn shutdown_signal_trigger(trigger: Trigger) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
    drop(trigger);
}

#[cfg(not(tarpaulin_include))]
///handle the shutdown signal
pub async fn shutdown_signal(shutdown: Tripwire) {
    shutdown.await;
}

#[cfg(not(tarpaulin_include))]
///handle fallback
pub async fn fallback(uri: Uri) -> (StatusCode, String) {
    error!("route not found: {uri}");
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
}
