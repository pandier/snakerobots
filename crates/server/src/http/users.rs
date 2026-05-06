use crate::http::error::{RouteError, RouteResult};
use crate::http::extract::AuthedUser;
use crate::service;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use snakerobots_shared::dto::user::UpdateCompetingRobot;
use snakerobots_shared::dto::{PrivateUser, User, UserRanking};
use snakerobots_shared::dto::game::{Match};
use uuid::Uuid;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}", get(get_user))
        .route("/{id}/matches", get(get_user_matches))
        .route("/{id}/ranking", get(get_user_ranking))
        .route("/me", get(get_me))
        .route("/me/competing-robot", post(update_competing_robot))
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

async fn get_user_matches(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> RouteResult<Json<Vec<Match>>> {
    let matches = service::matches::get_matches_by_user(&app, user_id)
        .await?
        .into_iter()
        .map(|m| m.into())
        .collect();
    Ok(Json(matches))
}

async fn get_user_ranking(
    State(app): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> RouteResult<Json<UserRanking>> {
    let rank = service::user::get_user_ranking(&app, user_id)
        .await?
        .ok_or_else(|| RouteError::not_found())?;
    Ok(Json(UserRanking { rank }))
}

async fn get_me(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
) -> RouteResult<Json<PrivateUser>> {
    Ok(Json(service::user::get_user(&app, user_id)
        .await?
        .ok_or_else(|| eyre::eyre!("authenticated user is missing from database"))?
        .into()))
}

async fn update_competing_robot(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Json(update): Json<UpdateCompetingRobot>,
) -> RouteResult<()> {
    let competing_robot_id = update.competing_robot_id
        .map(|s| Uuid::try_parse(&s))
        .transpose()
        .map_err(|_| RouteError::new(StatusCode::BAD_REQUEST, "invalid_id", "Invalid robot id"))?;

    if let Some(competing_robot_id) = &competing_robot_id {
        if !service::robot::exists_robot(&app, user_id, *competing_robot_id).await? {
            return Err(RouteError::not_found());
        }
    }

    service::user::update_competing_robot(&app, user_id, competing_robot_id).await?;
    Ok(())
}
