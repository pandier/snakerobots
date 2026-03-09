use chrono::{Duration, Utc};
use sqlx::types::Uuid;

use crate::{model::SessionModel, state::AppState};

pub async fn verify_session(app: &AppState, session_id: impl TryInto<Uuid>) -> eyre::Result<Option<SessionModel>> {
    Ok(get_session(app, session_id)
        .await?
        .filter(|x| Utc::now() < x.expires_at))
}

pub async fn get_session(app: &AppState, session_id: impl TryInto<Uuid>) -> eyre::Result<Option<SessionModel>> {
    let Ok(session_id) = session_id.try_into() else { return Ok(None); };
    Ok(sqlx::query_as("SELECT * FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn create_session(app: &AppState, user_id: Uuid) -> eyre::Result<SessionModel> {
    let expires_at = Utc::now() + Duration::days(30);
    Ok(sqlx::query_as("INSERT INTO sessions (user_id, expires_at) VALUES ($1, $2) RETURNING *")
        .bind(user_id)
        .bind(expires_at)
        .fetch_one(&app.pg)
        .await?)
}
