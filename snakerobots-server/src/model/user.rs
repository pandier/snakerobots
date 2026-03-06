use chrono::{DateTime, Utc};
use snakerobots_dto::UserDto;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserModel {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserModel> for UserDto {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id,
            username: value.username,
            created_at: value.created_at,
        }
    }
}
