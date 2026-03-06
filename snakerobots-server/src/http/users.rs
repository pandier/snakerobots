use crate::http::error::{RouteError, RouteResult};
use crate::service;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use snakerobots_dto::UserDto;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}", get(get_user))
}

async fn get_user(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> RouteResult<Json<UserDto>> {
    service::user::get_user(&app, user_id)
        .await?
        .ok_or_else(|| RouteError::not_found())
        .map(|user| Json(user.into()))
}
