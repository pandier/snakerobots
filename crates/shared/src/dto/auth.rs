use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::dto::PrivateUser;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"))]
    #[validate(custom(function = "super::util::validate_username_chars", message = "Username can only consist of a-z, 0-9 and _"))]
    pub username: String,
    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    #[validate(custom(function = "super::util::validate_non_control_chars", message = "Password contains illegal characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: PrivateUser,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 20, message = "Username must be between 3 and 20 characters"))]
    #[validate(custom(function = "super::util::validate_username_chars", message = "Username can only consist of a-z, 0-9 and _"))]
    pub username: String,
    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    #[validate(custom(function = "super::util::validate_non_control_chars", message = "Password contains illegal characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: PrivateUser,
    pub token: String,
}
