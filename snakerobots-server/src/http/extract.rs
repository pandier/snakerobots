use std::sync::Arc;

use axum::{extract::FromRequestParts, http::{StatusCode, header::AUTHORIZATION}};

use crate::{http::error::RouteError, service, state::AppState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthedUser(pub String);

impl FromRequestParts<Arc<AppState>> for AuthedUser
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        if let Some(authorization) = parts.headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
            if authorization.to_lowercase().starts_with("bearer ") {
                if let Some(session) = service::auth::verify_session(state, &authorization[7..]).await? {
                    return Ok(AuthedUser(session.user_id.to_string()));
                }
            }
        }
        Err(RouteError::new(StatusCode::UNAUTHORIZED, "unauthorized", "Unauthorized"))
    }
}
