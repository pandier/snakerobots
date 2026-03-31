pub mod dto;
pub mod logic;
pub mod math;

#[cfg(feature = "lang")]
pub use tropaion as lang;

pub use math::{Direction, Point, Size};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GameResult {
    Win { winner: usize },
    Tie,
    Abort,
}

impl GameResult {
    pub fn winner(&self) -> Option<usize> {
        match self {
            Self::Win { winner } => Some(*winner),
            _ => None,
        }
    }
}
