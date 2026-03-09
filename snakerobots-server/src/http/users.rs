use crate::http::error::{RouteError, RouteResult};
use crate::http::extract::AuthedUser;
use crate::service;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use eyre::Context;
use snakerobots_shared::dto::User;
use snakerobots_shared::dto::game::{CreateMatchRequest, Match, MatchRequest};
use sqlx::types::Uuid;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}", get(get_user))
        .route("/{id}/matches", get(get_user_matches))
        .route("/me", get(get_me))
        .route("/me/match-requests", get(get_match_requests))
        .route("/me/match-requests", post(create_match_request))
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
        .await
        .wrap_err("failed to get matches by user")?;
    let matches = futures::future::try_join_all(
        matches.into_iter().map(|m| service::matches::resolve_match(&app, m))
    ).await?;
    Ok(Json(matches))
}

async fn get_me(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
) -> RouteResult<Json<User>> {
    Ok(Json(service::user::get_user(&app, user_id)
        .await?
        .ok_or_else(|| eyre::eyre!("authenticated user is missing from database"))?
        .into()))
}

async fn get_match_requests(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
) -> RouteResult<Json<Vec<MatchRequest>>> {
    Ok(Json(service::matches::get_match_requests(&app, user_id)
        .await?
        .into_iter()
        .map(|m| m.into())
        .collect()))
}

async fn create_match_request(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Json(payload): Json<CreateMatchRequest>,
) -> RouteResult<Json<MatchRequest>> {
    let Ok(receiver_id) = Uuid::try_from(payload.receiver_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "bad_request", "This user does not exist"));
    };

    // TODO: Check if user exists
    let request = service::matches::create_match_request(&app, user_id, receiver_id).await?;

    if let Some(request) = request {
        Ok(Json(request.into()))
    } else {
        Err(RouteError::new(StatusCode::CONFLICT, "already_requested", "You have already requested a match with this user"))
    }
}
