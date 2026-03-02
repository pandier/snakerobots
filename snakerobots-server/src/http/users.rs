use crate::state::AppState;
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use snakerobots_dto::UserDto;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}", get(get_user))
}

async fn get_user(
    Path(user_id): Path<i32>,
) -> Json<UserDto> {
}
