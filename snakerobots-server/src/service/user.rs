use crate::{model::user::UserModel, state::AppState};

pub async fn get_user(app: &AppState, user_id: i32) -> eyre::Result<Option<UserModel>> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn create_user(app: &AppState, username: String, password: String) -> eyre::Result<Option<UserModel>> {
    Ok(sqlx::query_as("INSERT INTO users (username, password) VALUES ($1, $2) ON CONFLICT (username) DO NOTHING RETURNING *")
        .bind(username)
        .bind(password)
        .fetch_optional(&app.pg)
        .await?)
}
