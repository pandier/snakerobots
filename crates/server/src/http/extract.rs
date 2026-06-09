use std::sync::Arc;

use axum::{Json, extract::{FromRequest, FromRequestParts, Query, rejection::{JsonRejection, QueryRejection}}, http::{StatusCode, header::AUTHORIZATION}};
use serde::de::DeserializeOwned;
use sqlx::types::Uuid;
use validator::Validate;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedJson<T>(pub T);

impl<T> FromRequest<Arc<AppState>> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate
{
    type Rejection = Result<JsonRejection, RouteError>;

    async fn from_request(req: axum::extract::Request, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let value: Json<T> = Json::from_request(req, state).await
            .map_err(|e| Ok(e))?;

        value.0.validate()
            .map_err(|e| Err(RouteError::validation(Box::new(e))))?;

        Ok(Self(value.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedQuery<T>(pub T);

impl<T> FromRequestParts<Arc<AppState>> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate
{
    type Rejection = Result<QueryRejection, RouteError>;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let value: Query<T> = Query::from_request_parts(parts, state).await
            .map_err(|e| Ok(e))?;

        value.0.validate()
            .map_err(|e| Err(RouteError::validation(Box::new(e))))?;

        Ok(Self(value.0))
    }
}