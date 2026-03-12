use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::{delete, get, post}};
use snakerobots_shared::dto::match_request::{CreateMatchRequest, DeleteMatchRequest, MatchRequest};
use sqlx::types::Uuid;

use crate::{http::{error::{RouteError, RouteResult}, extract::AuthedUser}, service, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_match_requests))
        .route("/", post(create_match_request))
        .route("/", delete(delete_match_request))
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
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "bad_request", "Invalid id"));
    };

    // TODO: Check if user exists
    let request = service::matches::create_match_request(&app, user_id, receiver_id).await?;

    if let Some(request) = request {
        Ok(Json(request.into()))
    } else {
        Err(RouteError::new(StatusCode::CONFLICT, "already_requested", "You have already requested a match with this user"))
    }
}

async fn delete_match_request(
    State(app): State<Arc<AppState>>,
    AuthedUser(user_id): AuthedUser,
    Json(payload): Json<DeleteMatchRequest>,
) -> RouteResult<StatusCode> {
    let Ok(sender_id) = Uuid::try_from(payload.sender_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "bad_request", "Invalid id"));
    };
    let Ok(receiver_id) = Uuid::try_from(payload.receiver_id) else {
        return Err(RouteError::new(StatusCode::BAD_REQUEST, "bad_request", "Invalid id"));
    };
    if sender_id != user_id && receiver_id != user_id {
        return Err(RouteError::new(StatusCode::FORBIDDEN, "forbidden", "Cannot delete match requests that are not related to the authenticated user"));
    }

    let success = service::matches::delete_match_request(&app, sender_id, receiver_id).await?;

    if success {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}