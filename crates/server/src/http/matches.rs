use crate::http::error::{RouteError, RouteResult};
use crate::service;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use snakerobots_shared::dto::DefaultGameReplay;
use snakerobots_shared::dto::game::{Match};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}", get(get_match))
        .route("/{id}/replay", get(get_match_replay))
}

async fn get_match(
    State(app): State<Arc<AppState>>,
    Path(match_id): Path<String>,
) -> RouteResult<Json<Match>> {
    service::matches::get_match(&app, match_id)
        .await?
        .ok_or_else(|| RouteError::not_found())
        .map(|m| Json(m.into()))
}

async fn get_match_replay(
    State(app): State<Arc<AppState>>,
    Path(match_id): Path<String>,
) -> RouteResult<Json<DefaultGameReplay>> {
    service::matches::get_match_replay(&app, match_id)
        .await?
        .ok_or_else(|| RouteError::not_found())
        .map(|replay| Json(replay))
}