pub mod error;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use snakerobots_shared::dto::GameReplay;
use sqlx::{PgPool, types::{Json, chrono::{DateTime, Utc}}};
use uuid::Uuid;

use crate::queue::error::MatchQueueError;

type Result<T> = std::result::Result<T, MatchQueueError>;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct QueuedMatch {
    pub id: Uuid,
    pub details: QueuedMatchDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMatchDetails {
    pub ranked: bool,
    pub players: Vec<QueuedMatchDetailsPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMatchDetailsPlayer {
    pub id: Uuid,
    pub robot_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMatchResult {
    pub replay: Option<GameReplay<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct FinishedQueuedMatch {
    pub id: Uuid,
    pub queued_at: DateTime<Utc>,
    pub details: QueuedMatchDetails,
    pub result: QueuedMatchResult,
}

pub struct MatchQueue {
    pg: PgPool,
}

impl MatchQueue {
    pub fn new(pg: PgPool) -> MatchQueue {
        Self { pg }
    }

    pub async fn queue(&self, details: QueuedMatchDetails) -> Result<Uuid> {
        let (id,): (Uuid,) = sqlx::query_as(
            "INSERT INTO match_queue (details) VALUES ($1) RETURNING id"
        )
        .bind(Json(details))
        .fetch_one(&self.pg)
        .await?;

        Ok(id)
    }

    pub async fn take(&self, worker_id: Uuid, duration: Duration) -> Result<Option<QueuedMatch>> {
        let result: Option<(Uuid, Json<QueuedMatchDetails>)> = sqlx::query_as(r#"
            WITH next_match AS (
                SELECT id
                FROM match_queue
                WHERE result IS NULL AND (worker_id IS NULL OR expires_at IS NULL OR expires_at < now())
                ORDER BY queued_at
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            UPDATE match_queue
            SET worker_id = $1, expires_at = NOW() + $2
            WHERE id IN (SELECT id FROM next_match)
            RETURNING id, details
        "#)
        .bind(worker_id)
        .bind(duration)
        .fetch_optional(&self.pg)
        .await?;

        let mapped = result
            .map(|(id, Json(details))| QueuedMatch { id, details });

        Ok(mapped)
    }

    pub async fn update(&self, id: Uuid, worker_id: Uuid, duration: Duration) -> Result<bool> {
        let result = sqlx::query(r#"
            UPDATE match_queue
            SET expires_at = NOW() + $3
            WHERE id = $1 AND worker_id = $2
        "#)
        .bind(id)
        .bind(worker_id)
        .bind(duration)
        .execute(&self.pg)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn finish(&self, id: Uuid, worker_id: Uuid, result: QueuedMatchResult) -> Result<bool> {
        let result = sqlx::query(r#"
            UPDATE match_queue
            SET result = $3
            WHERE id = $1 AND worker_id = $2
        "#)
        .bind(id)
        .bind(worker_id)
        .bind(Json(result))
        .execute(&self.pg)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove(&self, id: Uuid, worker_id: Uuid) -> Result<bool> {
        let result = sqlx::query(r#"
            DELETE FROM match_queue
            WHERE id = $1 AND worker_id = $2
        "#)
        .bind(id)
        .bind(worker_id)
        .execute(&self.pg)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn take_finished(&self) -> Result<Vec<FinishedQueuedMatch>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            queued_at: DateTime<Utc>,
            details: Json<QueuedMatchDetails>,
            result: Json<QueuedMatchResult>,
        }

        let result: Vec<Row> = sqlx::query_as(r#"
            DELETE FROM match_queue
            WHERE result IS NOT NULL
            RETURNING id, queued_at, details, result
        "#)
        .fetch_all(&self.pg)
        .await?;

        let mapped = result.into_iter()
            .map(|row| FinishedQueuedMatch {
                id: row.id,
                queued_at: row.queued_at,
                details: row.details.0,
                result: row.result.0,
            })
            .collect();

        Ok(mapped)
    }
}
