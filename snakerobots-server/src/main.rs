pub mod http;
pub mod middleware;
pub mod model;
pub mod runner;
pub mod service;
pub mod state;

use crate::state::AppState;
use eyre::Context;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;

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

    tracing::info!("running migrations");

    sqlx::migrate!()
        .run(&state.pg)
        .await
        .wrap_err("failed to run migrations")?;

    http::serve(state).await
}
