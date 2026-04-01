use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::{delete, get, post}};
use eyre::Context;
use snakerobots_shared::dto::match_request::{AcceptMatchRequest, CreateMatchRequest, DeleteMatchRequest, MatchRequest};
use sqlx::types::Uuid;

use crate::{http::{error::{RouteError, RouteResult}, extract::AuthedUser}, service::{self, error::ServiceError}, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_match_requests))
        .route("/", post(create_match_request))
        .route("/", delete(delete_match_request))
        .route("/accept", post(accept_match_request))
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
    let Ok(robot_id) = Uuid::try_from(payload.robot_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "invalid_id", "Invalid id"));
    };

    let receiver = service::user::get_user_by_username(&app, &payload.username)
        .await
        .wrap_err("failed to get user by username")?
        .ok_or_else(|| RouteError::new(StatusCode::BAD_REQUEST, "unknown_user", "Couldn't find a user with this username"))?;

    if user_id == receiver.id {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "self_request", "Can't send a match request to yourself"));
    }

    if !service::robot::exists_robot(&app, user_id, robot_id).await? {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "unknown_robot", "The specified robot does not exist"));
    }

    let result = service::matches::create_match_request(&app, user_id, receiver.id, robot_id).await;
    match result {
        Ok(v) => Ok(Json(v.into())),
        Err(ServiceError::AlreadyExists(_)) => Err(RouteError::new(StatusCode::CONFLICT, "already_exists", "You already requested a match with this user")),
        Err(ServiceError::LimitReached(_)) => Err(RouteError::new(StatusCode::CONFLICT, "limit_reached", "You reached the limit for the number of match requests")),
        Err(e) => Err(e.into())
    }
}

async fn delete_match_request(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Json(payload): Json<DeleteMatchRequest>,
) -> RouteResult<StatusCode> {
    let Ok(sender_id) = Uuid::try_from(payload.sender_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "invalid_id", "Invalid id"));
    };
    let Ok(receiver_id) = Uuid::try_from(payload.receiver_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "invalid_id", "Invalid id"));
    };
    if sender_id != user_id && receiver_id != user_id {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "bad_request", "Match request must relate to the authenticated user"));
    }

    let success = service::matches::delete_match_request(&app, sender_id, receiver_id)
        .await?
        .is_some();

    if success {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

async fn accept_match_request(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Json(payload): Json<AcceptMatchRequest>,
) -> RouteResult<StatusCode> {
    let Ok(sender_id) = Uuid::try_from(payload.sender_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "invalid_id", "Invalid id"));
    };

    let robot_code = service::robot::download_robot(&app, user_id, payload.robot_id)
        .await?
        .ok_or_else(|| RouteError::new(StatusCode::BAD_REQUEST, "unknown_robot", "The specified robot does not exist"))?;

    let match_req = service::matches::delete_match_request(&app, sender_id, user_id)
        .await?
        .ok_or_else(|| RouteError::not_found())?;

    let sender_robot_id = match_req.sender_robot_id
        .ok_or_else(|| RouteError::new(StatusCode::CONFLICT, "robot_deleted", "One of the robots has been deleted since the creation of the match request"))?;

    let sender_robot_code = service::robot::download_robot(&app, sender_id, sender_robot_id).await?
        .ok_or_else(|| RouteError::new(StatusCode::CONFLICT, "robot_deleted", "One of the robots has been deleted since the creation of the match request"))?;

    service::game::queue_game(app.clone(), sender_id, sender_robot_code, user_id, robot_code);

    Ok(StatusCode::ACCEPTED)
}
