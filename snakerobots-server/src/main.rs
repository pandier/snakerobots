pub mod routes;
pub mod runner;
pub mod state;

use std::sync::Arc;

use eyre::Context;

use crate::state::AppState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let state = Arc::new(AppState {});
    let router = routes::router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .context("Failed to bind server")?;

    axum::serve(listener, router).await?;

    Ok(())
}
