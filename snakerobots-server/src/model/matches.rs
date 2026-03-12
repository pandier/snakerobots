use chrono::{DateTime, Utc};
use snakerobots_shared::{Direction, GameResult, dto::{MatchRequest, game::{Match, MatchPlayer}}};
use sqlx::{Row, postgres::PgRow, types::Uuid};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchModel {
    pub id: Uuid,
    pub seed: i64,
    pub winner_index: Option<i32>,
    pub aborted: bool,
    pub played_at: DateTime<Utc>,
}

impl MatchModel {
    pub fn into_dto(self, players: Vec<MatchPlayerModel>) -> Match {
        Match {
            id: self.id.to_string(),
            seed: self.seed as u64,
            players: players.into_iter().map(|p| MatchPlayer::from(p)).collect(),
            played_at: self.played_at,
            result: if self.aborted { GameResult::Abort } else { match self.winner_index {
                Some(winner) => GameResult::Win { winner: winner as usize },
                None => GameResult::Tie,
            }},
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchPlayerModel {
    pub user_id: Option<Uuid>,
    pub moves: Vec<Direction>,
}

impl From<MatchPlayerModel> for MatchPlayer {
    fn from(value: MatchPlayerModel) -> Self {
        Self {
            user_id: value.user_id.map(|u| u.to_string()),
            moves: value.moves,
        }
    }
}

impl sqlx::FromRow<'_, PgRow> for MatchPlayerModel {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            user_id: row.try_get("user_id")?,
            moves: Direction::try_vec_from_string(row.try_get("moves")?)
                .map_err(|e| sqlx::Error::ColumnDecode { index: "moves".into(), source: e.into() })?,
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchRequestModel {
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<MatchRequestModel> for MatchRequest {
    fn from(value: MatchRequestModel) -> Self {
        Self {
            sender_id: value.sender_id.to_string(),
            receiver_id: value.receiver_id.to_string(),
            created_at: value.created_at,
            expires_at: value.expires_at,
        }
    }
}
