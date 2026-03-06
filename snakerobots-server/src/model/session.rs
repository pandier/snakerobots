use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionModel {
    pub id: Uuid,
    pub user_id: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
