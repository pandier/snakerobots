pub mod dto;
pub mod logic;
pub mod math;

pub use math::{Direction, Point, Size};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "camelCase")]
pub enum GameResult {
    Win { winner: usize },
    Tie,
}
