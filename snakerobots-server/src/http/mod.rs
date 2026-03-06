mod auth;
mod error;
mod extract;
mod users;

use std::sync::Arc;

use axum::{Json, Router, routing::post};
use serde::Serialize;
use snakerobots_shared::dto;

use crate::{http::error::RouteResult, runner::run_game, state::AppState};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/run", post(run))
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct RunResponse {
    game: dto::Game,
}

async fn run() -> RouteResult<Json<RunResponse>> {
    let game = run_game().await.map_err(|r| eyre::Report::from(r))?;
    Ok(Json(RunResponse { game }))
}
