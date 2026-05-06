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
    pub elo: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateUser {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub elo: i32,
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
