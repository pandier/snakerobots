use rowplus::query_plus;
use uuid::Uuid;

use crate::{model::RobotModel, service::error::ServiceResult, state::AppState};

pub async fn list_robots(app: &AppState, user_id: &Uuid) -> ServiceResult<Vec<RobotModel>> {
    Ok(query_plus!(RobotModel, "SELECT {} FROM robots WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&app.pg)
        .await?)
}

pub async fn get_robot(app: &AppState, user_id: &Uuid, id: impl TryInto<Uuid>) -> ServiceResult<Option<RobotModel>> {
    let Ok(id) = id.try_into() else { return Ok(None); };
    Ok(query_plus!(RobotModel, "SELECT {} FROM robots WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn create_robot(app: &AppState, user_id: &Uuid, name: &str) -> ServiceResult<RobotModel> {
    Ok(query_plus!(RobotModel, "INSERT INTO robots (user_id, name) VALUES ($1, $2) RETURNING {}")
        .bind(user_id)
        .bind(name)
        .fetch_one(&app.pg)
        .await?)
}

pub async fn delete_robot(app: &AppState, user_id: &Uuid, id: impl TryInto<Uuid>) -> ServiceResult<bool> {
    let Ok(id) = id.try_into() else { return Ok(false); };
    let res = sqlx::query("DELETE FROM robots WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&app.pg)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn upload_robot(app: &AppState, user_id: &Uuid, id: impl TryInto<Uuid>, code: &str) -> ServiceResult<bool> {
    let Ok(id) = id.try_into() else { return Ok(false); };
    let res = sqlx::query("UPDATE robots SET code = $1, edited_at = NOW() WHERE id = $2 AND user_id = $3")
        .bind(code)
        .bind(id)
        .bind(user_id)
        .execute(&app.pg)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn download_robot(app: &AppState, user_id: &Uuid, id: impl TryInto<Uuid>) -> ServiceResult<Option<String>> {
    let Ok(id) = id.try_into() else { return Ok(None); };

    #[derive(rowplus_derive::RowPlus)]
    #[rowplus(alias = "robots")]
    struct Row {
        code: String,
    }

    Ok(query_plus!(Row, "SELECT {} FROM robots WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(&app.pg)
        .await?
        .map(|row| row.code))
}
