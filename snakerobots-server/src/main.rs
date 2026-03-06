pub mod model;
pub mod http;
pub mod runner;
pub mod service;
pub mod state;

use crate::state::AppState;
use eyre::Context;
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let state = Arc::new(AppState::new().await?);
    let router = http::router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("Failed to bind server")?;

    axum::serve(listener, router).await?;

    Ok(())
}
