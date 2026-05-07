use std::sync::Arc;

use anyhow::Context;
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

mod api;
mod auth;
mod chain;
mod config;
mod db;
mod embed;
mod error;
mod models;
mod validate;
mod ws;

use crate::api::AppState;
use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .compact()
        .init();

    let cfg = Config::from_env().context("loading config")?;
    tracing::info!(http = %cfg.http_bind, db = %cfg.db_url_redacted(), "starting indexer");

    let pool = db::connect(&cfg.database_url).await?;
    db::run_migrations(&pool).await?;

    let (event_tx, _) = broadcast::channel::<models::EventEnvelope>(256);

    let state = Arc::new(AppState {
        pool: pool.clone(),
        cfg: cfg.clone(),
        event_tx: event_tx.clone(),
    });

    if let Some(rpc_ws) = cfg.evm_ws_url.clone() {
        let watcher_state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = chain::evm::run(watcher_state, rpc_ws).await {
                tracing::error!(error = ?err, "evm watcher exited with error");
            }
        });
    } else {
        tracing::warn!("EVM_WS_URL not set; chain watcher is disabled (API still serves backfilled data)");
    }

    let app = api::router(state.clone());

    let listener = tokio::net::TcpListener::bind(cfg.http_bind).await?;
    tracing::info!("listening on {}", cfg.http_bind);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutting down");
}
