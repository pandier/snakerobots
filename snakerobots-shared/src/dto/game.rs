use serde::{Deserialize, Serialize};

use crate::{Direction, GameResult};

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
