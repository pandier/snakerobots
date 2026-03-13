use chrono::{Duration, Utc};
use eyre::Context;
use snakerobots_shared::{Direction, dto};
use sqlx::types::Uuid;

use crate::{model::{MatchModel, MatchPlayerModel, matches::MatchRequestModel}, state::AppState};

pub async fn get_match(app: &AppState, id: impl TryInto<Uuid>) -> eyre::Result<Option<MatchModel>> {
    let Ok(id) = id.try_into() else { return Ok(None); };
    Ok(sqlx::query_as("SELECT * FROM matches WHERE id = $1")
        .bind(id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn resolve_match(app: &AppState, match_: MatchModel) -> eyre::Result<dto::Match> {
    let players = get_match_players(&app, match_.id.clone()).await.wrap_err("failed to get match players")?;
    Ok(match_.into_dto(players))
}

pub async fn create_match(app: &AppState, seed: u64, winner: Option<usize>, snakes: Vec<(&dto::GameSnake, Uuid)>) -> eyre::Result<MatchModel> {
    let mut tx = app.pg.begin().await?;

    let model: MatchModel = sqlx::query_as("INSERT INTO matches (seed, winner_index, aborted) VALUES ($1, $2, $3) RETURNING *")
        .bind(seed as i64)
        .bind(winner.map(|index| index as i32))
        .bind(false)
        .fetch_one(&mut *tx)
        .await?;

    for (index, (snake, user_id)) in snakes.into_iter().enumerate() {
        sqlx::query("INSERT INTO match_players (\"index\", match_id, user_id, moves) VALUES ($1, $2, $3, $4)")
            .bind(index as i32)
            .bind(model.id)
            .bind(user_id)
            .bind(Direction::vec_to_string(&snake.moves))
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(model)
}

pub async fn get_match_players(app: &AppState, id: impl TryInto<Uuid>) -> eyre::Result<Vec<MatchPlayerModel>> {
    let Ok(id) = id.try_into() else { return Ok(vec![]); };
    Ok(sqlx::query_as("SELECT * FROM match_players WHERE match_id = $1")
        .bind(id)
        .fetch_all(&app.pg)
        .await?)
}

pub async fn get_matches_by_user(app: &AppState, user_id: impl TryInto<Uuid>) -> eyre::Result<Vec<MatchModel>> {
    let Ok(user_id) = user_id.try_into() else { return Ok(vec![]); };
    Ok(sqlx::query_as("SELECT m.* FROM matches m INNER JOIN match_players p ON m.id = p.match_id AND p.user_id = $1")
        .bind(user_id)
        .fetch_all(&app.pg)
        .await?)
}

pub async fn get_match_requests(app: &AppState, user_id: Uuid) -> eyre::Result<Vec<MatchRequestModel>> {
    Ok(sqlx::query_as("SELECT * FROM match_requests WHERE (sender_id = $1 OR receiver_id = $1) AND expires_at > now()")
        .bind(user_id)
        .fetch_all(&app.pg)
        .await?)
}

pub async fn create_match_request(app: &AppState, sender_id: Uuid, receiver_id: Uuid) -> eyre::Result<Option<MatchRequestModel>> {
    let expires_at = Utc::now() + Duration::minutes(30);
    // TODO: Check if the existing request has expired
    Ok(sqlx::query_as("INSERT INTO match_requests (sender_id, receiver_id, expires_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING RETURNING *")
        .bind(sender_id)
        .bind(receiver_id)
        .bind(expires_at)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn delete_match_request(app: &AppState, sender_id: Uuid, receiver_id: Uuid) -> eyre::Result<bool> {
    let result = sqlx::query("DELETE FROM match_requests WHERE sender_id = $1 AND receiver_id = $2")
        .bind(sender_id)
        .bind(receiver_id)
        .execute(&app.pg)
        .await?;
    Ok(result.rows_affected() > 0)
}
