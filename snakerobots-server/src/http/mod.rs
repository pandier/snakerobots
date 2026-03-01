mod users;

use std::sync::Arc;

use axum::{Json, Router, routing::post};
use serde::Serialize;
use snakerobots_dto::GameDto;

use crate::{runner::run_game, state::AppState};

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/run", post(run))
        .nest("/users", users::router())
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct RunResponse {
    game: GameDto,
}

async fn run() -> Json<RunResponse> {
    let game = run_game().await.unwrap(); // TODO: err handling
    Json(RunResponse { game })
}
