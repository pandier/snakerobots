use eyre::Context;
use sqlx::types::Uuid;
use tracing::debug;

use crate::{model::{UserModel, user::LeaderboardUserModel}, service, state::AppState};

pub async fn get_user(app: &AppState, user_id: impl TryInto<Uuid>) -> eyre::Result<Option<UserModel>> {
    let Ok(user_id) = user_id.try_into() else { return Ok(None); };
    Ok(sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn get_user_by_username(app: &AppState, username: &str) -> eyre::Result<Option<UserModel>> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(&app.pg)
        .await?)
}

pub async fn get_user_ranking(app: &AppState, user_id: impl TryInto<Uuid>) -> eyre::Result<Option<i64>> {
    let Ok(user_id) = user_id.try_into() else { return Ok(None); };
    let Some((elo,)): Option<(i32,)> = sqlx::query_as("SELECT elo FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&app.pg)
        .await? else { return Ok(None) };
    let (rank,): (i64,) = sqlx::query_as("SELECT COUNT(*) + 1 AS rank FROM users WHERE elo > $1 AND ranked")
        .bind(elo)
        .fetch_one(&app.pg)
        .await?;
    Ok(Some(rank))
}

pub async fn update_competing_robot(app: &AppState, id: Uuid, robot_id: Option<Uuid>) -> eyre::Result<()> {
    sqlx::query("UPDATE users SET competing_robot_id = $1 WHERE id = $2")
        .bind(robot_id)
        .bind(id)
        .execute(&app.pg)
        .await?;
    Ok(())
}

pub async fn create_user(app: &AppState, username: String, password: String) -> eyre::Result<Option<UserModel>> {
    let hashed_password = service::crypto::hash_password(password)
        .await
        .wrap_err("failed to hash password")?;
    Ok(sqlx::query_as("INSERT INTO users (username, password) VALUES ($1, $2) ON CONFLICT (username) DO NOTHING RETURNING *")
        .bind(username)
        .bind(hashed_password)
        .fetch_optional(&app.pg)
        .await?)
}

const ELO_K: f64 = 20f64;

pub async fn update_elo(app: &AppState, user1: Uuid, user2: Uuid, winner: Option<Uuid>) -> eyre::Result<((f64, f64), (f64, f64))> {
    let mut tx = app.pg.begin().await?;

    let (elo1,): (f64,) = sqlx::query_as("SELECT elo FROM users WHERE id = $1 FOR UPDATE")
        .bind(user1)
        .fetch_one(&mut *tx)
        .await?;

    let (elo2,): (f64,) = sqlx::query_as("SELECT elo FROM users WHERE id = $1 FOR UPDATE")
        .bind(user2)
        .fetch_one(&mut *tx)
        .await?;

    // calculate the diff for user 1
    let score = winner.map(|w| if w == user1 { 1f64 } else { 0f64 }).unwrap_or(0.5f64);
    let expected = 1f64 / (1f64 + 10f64.powf((elo2 - elo1) / 400f64));
    let diff = ELO_K * (score - expected);

    let new_elo1 = elo1 + diff;
    let new_elo2 = elo2 - diff;

    sqlx::query("UPDATE users SET elo = $1, ranked = TRUE WHERE id = $2")
        .bind(new_elo1)
        .bind(user1)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE users SET elo = $1, ranked = TRUE WHERE id = $2")
        .bind(new_elo2)
        .bind(user2)
        .execute(&mut *tx)
        .await?;

    debug!("ranked match between {user1} ({elo1}) and {user2} ({elo2}) resulted in score {score}, diff {diff}");

    tx.commit().await?;

    Ok(((elo1, diff), (elo2, -diff)))
}

pub async fn get_users_for_matchmaking(app: &AppState, offset: i32, limit: i32) -> eyre::Result<Vec<(Uuid, f64, Uuid)>> {
    Ok(sqlx::query_as("SELECT id, elo, competing_robot_id FROM users WHERE competing_robot_id IS NOT NULL LIMIT $1 OFFSET $2")
        .bind(limit)
        .bind(offset)
        .fetch_all(&app.pg)
        .await?)
}

pub async fn get_user_leaderboard(app: &AppState, offset: i64, limit: i64) -> eyre::Result<Vec<LeaderboardUserModel>> {
    Ok(sqlx::query_as("SELECT id, username, elo, RANK() OVER (ORDER BY elo DESC, id) AS rank FROM users WHERE ranked LIMIT $1 OFFSET $2")
        .bind(limit)
        .bind(offset)
        .fetch_all(&app.pg)
        .await?)
}
