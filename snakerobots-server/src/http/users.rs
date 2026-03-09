use crate::http::error::{RouteError, RouteResult};
use crate::http::extract::AuthedUser;
use crate::service;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use eyre::ContextCompat;
use snakerobots_shared::dto::User;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}", get(get_user))
        .route("/me", get(get_me))
}

async fn get_user(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> RouteResult<Json<User>> {
    service::user::get_user(&app, user_id)
        .await?
        .ok_or_else(|| RouteError::not_found())
        .map(|user| Json(user.into()))
}

async fn get_me(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
) -> RouteResult<Json<User>> {
    Ok(Json(service::user::get_user(&app, user_id)
        .await?
        .wrap_err("authenticated user is missing from database")?
        .into()))
}
