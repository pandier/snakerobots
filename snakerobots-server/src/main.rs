pub mod http;
pub mod middleware;
pub mod model;
pub mod service;
pub mod state;

use crate::state::{AppState, env_optional};
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

    let state = Arc::new(AppState::new().await?);

    if env_optional("MIGRATE")?.unwrap_or(true) {
        info!("running migrations");

        sqlx::migrate!()
            .run(&state.pg)
            .await
            .wrap_err("failed to run migrations")?;
    }

    tokio::spawn(cleanup_task(state.clone()));

    http::serve(state).await
}

async fn cleanup_task(app: Arc<AppState>) {
    loop {
        sleep(Duration::from_mins(30)).await;
        debug!("executing cleanups");
        let _ = service::auth::cleanup_sessions(&app)
            .await
            .inspect_err(|err| tracing::error!("failed to cleanup sessions: {}", err));
        let _ = service::matches::cleanup_match_requests(&app)
            .await
            .inspect_err(|err| tracing::error!("failed to cleanup match requests: {}", err));
    }
}
