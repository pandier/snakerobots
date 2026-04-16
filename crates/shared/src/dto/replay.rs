use serde::{Deserialize, Serialize};

use crate::{Direction, GameResult, logic::{Game, GameStep, Robot, robot::replay::ReplayRobot, standard::create_standard_game}};

pub type DefaultGameReplay = GameReplay<Option<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnakeReplay<M> {
    #[serde(with = "super::util::directions")]
    pub moves: Vec<Direction>,
    pub metadata: M,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameReplay<M> {
    pub seed: u64,
    pub result: GameResult,
    pub snakes: Vec<SnakeReplay<M>>,
}

impl<M> GameReplay<M> {

    pub fn run_standard(robot1: Box<dyn Robot>, metadata1: M, robot2: Box<dyn Robot>, metadata2: M) -> Self {
        let mut game = create_standard_game(robot1, Some(robot2), None);

        let mut snakes = vec![
            SnakeReplay {
                moves: Vec::new(),
                metadata: metadata1,
            },
            SnakeReplay {
                moves: Vec::new(),
                metadata: metadata2,
            },
        ];

        let result = loop {
            match game.step_infallible() {
                GameStep::Success { moves, added_apples: _, removed_apples: _ } => {
                    for (i, dir) in moves {
                        if let Some(snake) = snakes.get_mut(i) {
                            snake.moves.push(dir);
                        }
                    }
                }
                GameStep::Finished(result) => break result,
            }
        };

        Self {
            seed: game.seed(),
            result,
            snakes,
        }
    }

    pub fn create_game(&self) -> Game {
        let robot1 = ReplayRobot::new(self.snakes[0].moves.clone());
        let robot2 = ReplayRobot::new(self.snakes[1].moves.clone());
        create_standard_game(Box::new(robot1), Some(Box::new(robot2)), Some(self.seed))
    }

    pub fn winner(&self) -> Option<&SnakeReplay<M>> {
        self.result.winner().and_then(|i| self.snakes.get(i))
    }
}
