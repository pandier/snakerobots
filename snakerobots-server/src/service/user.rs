use eyre::Context;

use crate::{model::user::UserModel, service, state::AppState};

pub async fn get_user(app: &AppState, user_id: i32) -> eyre::Result<Option<UserModel>> {
    Ok(sqlx::query_as("SELECTA * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn get_user_by_username(app: &AppState, username: &str) -> eyre::Result<Option<UserModel>> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn create_user(app: &AppState, username: String, password: String) -> eyre::Result<Option<UserModel>> {
    let hashed_password = service::crypto::hash_password(password)
        .await
        .wrap_err("failed to hash password")?;
    Ok(sqlx::query_as("INSERT INTO users (username, password) VALUES ($1, $2) ON CONFLICT (username) DO NOTHING RETURNING *")
        .bind(username)
        .bind(hashed_password)
        .fetch_optional(&app.pg)
        .await?)
}
