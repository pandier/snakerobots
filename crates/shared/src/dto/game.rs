use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Direction, GameResult, dto::User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub players: Vec<Option<User>>,
    pub played_at: DateTime<Utc>,
    pub winner: Option<String>,
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
