use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub receiver_id: String,
    pub sender_id: String,
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
