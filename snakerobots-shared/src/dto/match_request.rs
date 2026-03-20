use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dto::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub receiver: User,
    pub sender: User,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMatchRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMatchRequest {
    pub sender_id: String,
    pub receiver_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptMatchRequest {
    pub sender_id: String,
}
