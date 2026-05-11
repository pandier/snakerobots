use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
pub struct LeaderboardQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardUser {
    pub id: String,
    pub username: String,
    pub elo: f64,
    pub rank: i64,
}
