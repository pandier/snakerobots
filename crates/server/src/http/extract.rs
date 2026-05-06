use std::sync::Arc;

use axum::{extract::FromRequestParts, http::{StatusCode, header::AUTHORIZATION}};
use sqlx::types::Uuid;

use crate::{http::error::RouteError, model::SessionModel, service, state::AppState};

#[derive(Debug, Clone)]
pub struct AuthedSession(pub SessionModel);

impl FromRequestParts<Arc<AppState>> for AuthedSession
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        if let Some(authorization) = parts.headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
            if authorization.to_lowercase().starts_with("bearer ") {
                if let Some(session) = service::auth::verify_session(state, &authorization[7..]).await? {
                    return Ok(AuthedSession(session));
                }
            }
        }
        Err(RouteError::new(StatusCode::UNAUTHORIZED, "unauthorized", "Unauthorized"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthedUser(pub Uuid);

impl FromRequestParts<Arc<AppState>> for AuthedUser
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let session = AuthedSession::from_request_parts(parts, state).await?;
        Ok(AuthedUser(session.0.user_id))
    }
}
