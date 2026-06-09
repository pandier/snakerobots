use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRobot {
    #[validate(length(min = 1, max = 64, message = "Name must be between 1 and 64 characters"))]
    #[validate(custom(function = "super::util::validate_non_control_chars", message = "Name contains illegal characters"))]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RenameRobot {
    #[validate(length(min = 1, max = 64, message = "Name must be between 1 and 64 characters"))]
    #[validate(custom(function = "super::util::validate_non_control_chars", message = "Name contains illegal characters"))]
    pub name: String
}
