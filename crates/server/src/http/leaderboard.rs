use std::sync::Arc;

use axum::{Json, Router, extract::{Query, State}, routing::get};
use snakerobots_shared::dto::{LeaderboardUser, user::LeaderboardQuery};

use crate::{http::error::RouteResult, service, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_leaderboard))
}

async fn get_leaderboard(
    State(app): State<Arc<AppState>>,
    Query(query): Query<LeaderboardQuery>,
) -> RouteResult<Json<Vec<LeaderboardUser>>> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20);
    let users = service::user::get_user_leaderboard(&app, offset, limit)
        .await?
        .into_iter()
        .map(|u| u.into())
        .collect();
    Ok(Json(users))
}
