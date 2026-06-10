use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dto::Match;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortUser {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub elo: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateUser {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub elo: f64,
    pub competing_robot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRanking {
    pub rank: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCompetingRobot {
    pub competing_robot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchesResponse {
    pub total: i64,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserMatchesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 0, message = "Offset cannot be negative"))]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LeaderboardQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 0, message = "Offset cannot be negative"))]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub total: i64,
    pub users: Vec<LeaderboardUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardUser {
    pub id: String,
    pub username: String,
    pub elo: f64,
    pub rank: i64,
}
