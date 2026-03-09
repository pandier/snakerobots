use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Direction, GameResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub seed: u64,
    pub players: Vec<MatchPlayer>,
    pub played_at: DateTime<Utc>,
    pub result: GameResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPlayer {
    pub user_id: Option<String>,
    #[serde(with = "crate::dto::util::directions")]
    pub moves: Vec<Direction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub receiver_id: String,
    pub sender_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMatchRequest {
    pub receiver_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(with = "crate::dto::util::Hex")]
    pub seed: u64,
    pub snakes: Vec<GameSnake>,
    #[serde(flatten)]
    pub result: GameResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnake {
    #[serde(with = "crate::dto::util::directions")]
    pub moves: Vec<Direction>,
}
