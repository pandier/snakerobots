use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use snakerobots_dto::{UserDto, auth::RegisterRequest};

use crate::{http::error::{RouteError, RouteResult}, service, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
}

async fn register(
    State(app): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> RouteResult<Json<UserDto>> {
    let user = service::user::create_user(&app, payload.username, payload.password)
        .await?
        .map(|user| user.into());
    if let Some(user) = user {
        Ok(Json(user))
    } else {
        Err(RouteError::new(StatusCode::CONFLICT, "username_taken", "This username is already taken."))
    }
}

