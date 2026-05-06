use serde::{Deserialize, Serialize};

use crate::dto::PrivateUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    #[serde(deserialize_with = "super::util::username")]
    pub username: String,
    #[serde(deserialize_with = "super::util::password")]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user: PrivateUser,
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
    pub user: PrivateUser,
    pub token: String,
}
