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
    #[serde(with = "super::util::directions")]
    pub moves: Vec<Direction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub seed: u64,
    pub snakes: Vec<GameSnake>,
    #[serde(flatten)]
    pub result: GameResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnake {
    #[serde(with = "super::util::directions")]
    pub moves: Vec<Direction>,
}
