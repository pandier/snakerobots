use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDto {
    #[serde(with = "crate::util::Hex")]
    pub seed: u64,
    pub snakes: Vec<GameSnakeDto>,
    #[serde(flatten)]
    pub result: GameResultDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "camelCase")]
pub enum GameResultDto {
    Win { winner: usize },
    Tie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnakeDto {
    #[serde(with = "crate::util::moves")]
    pub moves: Vec<GameMoveDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMoveDto {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for GameMoveDto {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'u' | 'U' => Ok(Self::Up),
            'd' | 'D' => Ok(Self::Down),
            'l' | 'L' => Ok(Self::Left),
            'r' | 'R' => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

impl From<GameMoveDto> for char {
    fn from(value: GameMoveDto) -> Self {
        match value {
            GameMoveDto::Up => 'u',
            GameMoveDto::Down => 'd',
            GameMoveDto::Left => 'l',
            GameMoveDto::Right => 'r',
        }
    }
}
