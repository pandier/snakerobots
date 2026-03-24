use chrono::{DateTime, Utc};
use rowplus_derive::RowPlus;
use serde::Deserialize;
use snakerobots_shared::dto;
use uuid::Uuid;

#[derive(Debug, Clone, RowPlus)]
#[rowplus(alias = "users")]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserModel> for dto::User {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            created_at: value.created_at,
        }
    }

}
#[derive(Debug, Clone, RowPlus, Deserialize)]
#[rowplus(alias = "users")]
pub struct PartialUserModel {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl From<PartialUserModel> for dto::User {
    fn from(value: PartialUserModel) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            created_at: value.created_at,
        }
    }
}
