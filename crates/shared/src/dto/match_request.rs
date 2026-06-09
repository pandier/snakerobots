use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dto::ShortUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub receiver: ShortUser,
    pub sender: ShortUser,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateMatchRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"))]
    #[validate(custom(function = "super::util::validate_username_chars", message = "Username can only consist of a-z, 0-9 and _"))]
    pub username: String,
    pub robot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMatchRequest {
    pub sender_id: String,
    pub receiver_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptMatchRequest {
    pub sender_id: String,
    pub robot_id: String,
}
