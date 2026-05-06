use std::collections::HashMap;

use chrono::{Duration, Utc};
use rowplus::query_plus;
use rowplus_derive::RowPlus;
use snakerobots_shared::dto;
use sqlx::types::{Json, Uuid};

use crate::{model::{MatchModel, MatchPlayerModel, MatchWithPlayersModel, matches::{DeletedMatchRequestModel, MatchRequestModel}}, service::error::ServiceResult, state::AppState};

#[derive(Debug, Clone, RowPlus)]
#[rowplus(alias = "match_players")]
struct PlayerWithMatchRow {
    #[rowplus(flatten)]
    pub player: MatchPlayerModel,
    #[rowplus(nested, alias = "match")]
    pub match_: MatchModel,
}

pub async fn get_match(app: &AppState, id: impl TryInto<Uuid>) -> ServiceResult<Option<MatchWithPlayersModel>> {
    let Ok(id) = id.try_into() else { return Ok(None); };

    let rows = query_plus!(
        PlayerWithMatchRow,
        r#"
            SELECT {} FROM matches match
            JOIN match_players ON match_players.match_id = match.id AND match.id = $1
            JOIN users "user" ON "user".id = match_players.user_id
        "#
    )
    .bind(id)
    .fetch_all(&app.pg)
    .await?;

    let mut result: Option<MatchWithPlayersModel> = None;

    for row in rows {
        if let Some(match_) = &mut result {
            match_.players.push(row.player);
            continue;
        }
        result = Some(row.match_.with_players(vec![row.player]));
    }

    Ok(result)
}

pub async fn get_matches_by_user(app: &AppState, user_id: impl TryInto<Uuid>) -> ServiceResult<Vec<MatchWithPlayersModel>> {
    let Ok(user_id) = user_id.try_into() else { return Ok(vec![]); };
    let rows = query_plus!(
        PlayerWithMatchRow,
        r#"
            SELECT {} FROM match_players filter
            JOIN matches "match" ON "match".id = filter.match_id
            JOIN match_players ON match_players.match_id = "match".id
            JOIN users "user" ON "user".id = match_players.user_id
            WHERE filter.user_id = $1
            ORDER BY "match".id
        "#
    )
    .bind(user_id)
    .fetch_all(&app.pg)
    .await?;

    let mut result: Vec<MatchWithPlayersModel> = Vec::new();

    for row in rows {
        if let Some(last) = result.last_mut() {
            if last.match_.id == row.match_.id {
                last.players.push(row.player);
                continue;
            }
        }
        result.push(row.match_.with_players(vec![row.player]));
    }

    Ok(result)
}

pub async fn get_match_replay(app: &AppState, id: impl TryInto<Uuid>) -> ServiceResult<Option<dto::DefaultGameReplay>> {
    let Ok(id) = id.try_into() else { return Ok(None); };

    #[derive(sqlx::FromRow)]
    struct Row {
        replay: Json<dto::DefaultGameReplay>,
    }

    let row: Option<Row> = sqlx::query_as("SELECT replay FROM matches WHERE id = $1")
        .bind(id)
        .fetch_optional(&app.pg)
        .await?;

    Ok(row.map(|row| row.replay.0))
} 

pub async fn create_match(app: &AppState, replay: dto::GameReplay<Option<Uuid>>, elo: HashMap<Uuid, (f64, f64)>) -> ServiceResult<()> {
    let mut tx = app.pg.begin().await?;

    let winner = replay.winner().and_then(|snake| snake.metadata);

    let model = query_plus!(
        MatchModel,
        "INSERT INTO matches (winner, aborted, replay, ranked) VALUES ($1, $2, $3, $4) RETURNING {}"
    )
    .bind(winner)
    .bind(false)
    .bind(Json(replay.clone()))
    .bind(!elo.is_empty())
    .fetch_one(&mut *tx)
    .await?;

    for snake in replay.snakes {
        if let Some(user_id) = snake.metadata {
            let (elo, elo_diff) = match elo.get(&user_id) {
                Some((elo, elo_diff)) => (Some(*elo), Some(*elo_diff)),
                None => (None, None),
            };
            sqlx::query("INSERT INTO match_players (match_id, user_id, elo, elo_diff) VALUES ($1, $2, $3, $4)")
                .bind(model.id)
                .bind(user_id)
                .bind(elo)
                .bind(elo_diff)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

pub async fn get_match_requests(app: &AppState, user_id: Uuid) -> eyre::Result<Vec<MatchRequestModel>> {
    Ok(
        query_plus!(
            MatchRequestModel,
            r#"
                SELECT {} FROM match_requests
                INNER JOIN users sender ON sender.id = match_requests.sender_id
                INNER JOIN users receiver ON receiver.id = match_requests.receiver_id
                WHERE (match_requests.sender_id = $1 OR match_requests.receiver_id = $1) AND match_requests.expires_at > now()
            "#
        )
        .bind(user_id)
        .fetch_all(&app.pg)
        .await?
    )
}

pub async fn create_match_request(app: &AppState, sender_id: Uuid, receiver_id: Uuid, robot_id: Uuid) -> ServiceResult<MatchRequestModel> {
    let expires_at = Utc::now() + Duration::minutes(30);

    Ok(
        query_plus!(
            MatchRequestModel,
            r#"
                WITH inserted AS (
                    INSERT INTO match_requests (sender_id, sender_robot_id, receiver_id, expires_at) 
                    VALUES ($1, $2, $3, $4)
                    RETURNING *
                )
                SELECT {} FROM inserted match_requests
                INNER JOIN users sender ON sender.id = match_requests.sender_id
                INNER JOIN users receiver ON receiver.id = match_requests.receiver_id
            "#
        )
        .bind(sender_id)
        .bind(robot_id)
        .bind(receiver_id)
        .bind(expires_at)
        .fetch_one(&app.pg)
        .await?
    )
}

pub async fn delete_match_request(app: &AppState, sender_id: Uuid, receiver_id: Uuid) -> eyre::Result<Option<DeletedMatchRequestModel>> {
    Ok(query_plus!(DeletedMatchRequestModel, "DELETE FROM match_requests WHERE sender_id = $1 AND receiver_id = $2 RETURNING {}")
        .bind(sender_id)
        .bind(receiver_id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn cleanup_match_requests(app: &AppState) -> ServiceResult<()> {
    sqlx::query("DELETE FROM match_requests WHERE expires_at <= now()")
        .execute(&app.pg)
        .await?;

    Ok(())
}
