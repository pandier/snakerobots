use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use eyre::Context;
use snakerobots_shared::dto::auth::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse};
use tracing::info;

use crate::{http::{error::{RouteError, RouteResult}, extract::{AuthedSession, ValidatedJson}}, service, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

async fn register(
    State(app): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> RouteResult<Json<RegisterResponse>> {
    let user = service::user::create_user(&app, payload.username, payload.password)
        .await?;
    if let Some(user) = user {
        let session = service::auth::create_session(&app, user.id).await?;
        info!(user_id = ?user.id, sesion_id = ?session.id, username = &user.username, "register");
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
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> RouteResult<Json<LoginResponse>> {
    let user = service::user::get_user_by_username(&app, &payload.username)
        .await?;
    if let Some(user) = user {
        let verified = service::crypto::verify_password(payload.password, user.password.clone())
            .await
            .wrap_err("failed to verify password")?;
        if verified {
            let session = service::auth::create_session(&app, user.id).await?;
            info!(user_id = ?user.id, session_id = ?session.id, expires_at = ?session.expires_at, "login");
            return Ok(Json(LoginResponse {
                user: user.into(),
                token: session.id.to_string(),
            }));
        }
    }
    Err(RouteError::new(StatusCode::UNAUTHORIZED, "invalid_credentials", "Invalid username or password"))
}

async fn logout(
    State(app): State<Arc<AppState>>,
    AuthedSession(session): AuthedSession,
) -> RouteResult<()> {
    service::auth::remove_session(&app, session.id).await?;
    info!(user_id = ?session.user_id, session_id = ?session.id, "logout");
    Ok(())
}
