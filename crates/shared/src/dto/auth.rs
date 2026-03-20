use serde::{Deserialize, Serialize};

use crate::dto::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    #[serde(deserialize_with = "super::util::username")]
    pub username: String,
    #[serde(deserialize_with = "super::util::password")]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    #[serde(deserialize_with = "super::util::username")]
    pub username: String,
    #[serde(deserialize_with = "super::util::password")]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
}
