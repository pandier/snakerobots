mod auth;
mod error;
mod extract;
mod leaderboard;
mod match_requests;
mod matches;
mod robots;
mod users;

use std::sync::Arc;

use axum::Router;
use eyre::Context;
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};
use tracing::info;

use crate::{middleware::DevTokenHeader, state::AppState};

pub async fn serve(state: Arc<AppState>) -> eyre::Result<()> {
    let shutdown = state.shutdown.clone();

    let router = router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .wrap_err("failed to bind server")?;

    info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { shutdown.cancelled().await })
        .await?;

    Ok(())
}

fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/matches", matches::router())
        .nest("/match-requests", match_requests::router())
        .nest("/robots", robots::router())
        .nest("/leaderboard", leaderboard::router())
        .layer(tower::util::option_layer(state.dev_token.clone().map(|dev_token| ValidateRequestHeaderLayer::custom(DevTokenHeader(dev_token)))))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http()))
        .with_state(state)
}
