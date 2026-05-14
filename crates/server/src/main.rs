pub mod http;
pub mod middleware;
pub mod model;
pub mod service;
pub mod state;
pub mod matchmaking;

use crate::{matchmaking::Matchmaker, state::{AppState, env_optional}};
use eyre::Context;
use tokio::time::sleep;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let app = Arc::new(AppState::new().await?);

    if env_optional("MIGRATE")?.unwrap_or(false) {
        info!("running migrations");

        sqlx::migrate!()
            .run(&app.pg)
            .await
            .wrap_err("failed to run migrations")?;
    }

    tokio::spawn(cleanup_task(app.clone()));
    tokio::spawn(match_queue_task(app.clone()));

    let matchmaker = Matchmaker::new(app.clone());
    tokio::spawn(async move { matchmaker.start().await });

    http::serve(app).await
}

async fn cleanup_task(app: Arc<AppState>) {
    loop {
        debug!("executing cleanups");

        let _ = service::auth::cleanup_sessions(&app)
            .await
            .inspect_err(|err| tracing::error!("failed to cleanup sessions: {}", err));
        let _ = service::matches::cleanup_match_requests(&app)
            .await
            .inspect_err(|err| tracing::error!("failed to cleanup match requests: {}", err));

        tokio::select! {
            _ = sleep(Duration::from_mins(30)) => {},
            _ = app.shutdown.cancelled() => break,
        }
    }
}

async fn match_queue_task(app: Arc<AppState>) {
    loop {
        debug!("finalizing queued matches");

        let _ = service::match_queue::finalize_queued_matches(&app)
            .await
            .inspect_err(|err| tracing::error!("failed to finalize queued matches: {}", err));

        tokio::select! {
            _ = sleep(Duration::from_secs(20)) => {},
            _ = app.shutdown.cancelled() => break,
        }
    }
}
