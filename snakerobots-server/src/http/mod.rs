mod auth;
mod error;
mod extract;
mod match_requests;
mod users;

use std::sync::Arc;

use axum::Router;
use eyre::Context;
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, validate_request::ValidateRequestHeaderLayer};

use crate::{middleware::dev_token::DevTokenHeader, state::AppState};

pub async fn serve(state: Arc<AppState>) -> eyre::Result<()> {
    let router = router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .wrap_err("failed to bind server")?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, router).await?;

    Ok(())
}

fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/match-requests", match_requests::router())
        .layer(tower::util::option_layer(state.dev_token.clone().map(|dev_token| ValidateRequestHeaderLayer::custom(DevTokenHeader(dev_token)))))
        .layer(ServiceBuilder::new()
            .layer(TraceLayer::new_for_http()))
        .with_state(state)
}
