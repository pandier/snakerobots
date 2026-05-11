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
    pub elo: f64,
    pub competing_robot_id: Option<Uuid>,
}

impl From<UserModel> for dto::User {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            created_at: value.created_at,
            elo: value.elo,
        }
    }
}

impl From<UserModel> for dto::PrivateUser {
    fn from(value: UserModel) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            created_at: value.created_at,
            elo: value.elo,
            competing_robot_id: value.competing_robot_id.map(|x| x.to_string()),
        }
    }
}

#[derive(Debug, Clone, RowPlus, Deserialize)]
#[rowplus(alias = "users")]
pub struct PartialUserModel {
    pub id: Uuid,
    pub username: String,
}

impl From<PartialUserModel> for dto::ShortUser {
    fn from(value: PartialUserModel) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LeaderboardUserModel {
    pub id: Uuid,
    pub username: String,
    pub elo: f64,
    pub rank: i64,
}

impl From<LeaderboardUserModel> for dto::LeaderboardUser {
    fn from(value: LeaderboardUserModel) -> Self {
        Self {
            id: value.id.to_string(),
            username: value.username,
            elo: value.elo,
            rank: value.rank,
        }
    }
}
