use chrono::{DateTime, Utc};
use snakerobots_shared::dto;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserModel> for dto::User {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id,
            username: value.username,
            created_at: value.created_at,
        }
    }
}
