use chrono::{DateTime, Utc};
use rowplus_derive::RowPlus;
use snakerobots_shared::dto;
use uuid::Uuid;

#[derive(Debug, Clone, RowPlus)]
#[rowplus(alias = "robots")]
pub struct RobotModel {
    id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
    edited_at: DateTime<Utc>,
}

impl From<RobotModel> for dto::Robot {
    fn from(value: RobotModel) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            created_at: value.created_at,
            edited_at: value.edited_at,
        }
    }
}
