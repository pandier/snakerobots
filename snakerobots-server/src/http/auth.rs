use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use eyre::Context;
use snakerobots_shared::dto::auth::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use tracing::info;

use crate::{http::error::{RouteError, RouteResult}, service, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
    State(app): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> RouteResult<Json<RegisterResponse>> {
    let user = service::user::create_user(&app, payload.username, payload.password)
        .await?;
    if let Some(user) = user {
        let session = service::auth::create_session(&app, user.id).await?;
        info!(user_id=user.id.to_string(), username=&user.username, "register");
        Ok(Json(RegisterResponse {
            user: user.into(),
            token: session.id.to_string(),
        }))
    } else {
        Err(RouteError::new(StatusCode::CONFLICT, "username_taken", "This username is already taken"))
    }
}

async fn login(
    State(app): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> RouteResult<Json<LoginResponse>> {
    let user = service::user::get_user_by_username(&app, &payload.username)
        .await?;
    if let Some(user) = user {
        let verified = service::crypto::verify_password(payload.password, user.password.clone())
            .await
            .wrap_err("failed to verify password")?;
        if verified {
            let session = service::auth::create_session(&app, user.id).await?;
            info!(user_id=user.id.to_string(), expires_at=session.expires_at.to_string(), "login");
            return Ok(Json(LoginResponse {
                user: user.into(),
                token: session.id.to_string(),
            }));
        }
    }
    Err(RouteError::new(StatusCode::UNAUTHORIZED, "invalid_credentials", "Invalid username or password"))
}
