use serde::{Deserialize, Serialize};

use crate::{Direction, GameResult, logic::{Game, GameStep, robot::{impls::PathfindRobot, replay::ReplayRobot}, standard::create_standard_game}};

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

    pub fn run_standard<F>(metadata_factory: F) -> Self
    where
        F: Fn(usize) -> M,
    {
        let mut game = create_standard_game(|_| Box::new(PathfindRobot::new()));

        let mut snakes: Vec<SnakeReplay<M>> = game.players().iter()
            .enumerate()
            .map(|(i, _)| SnakeReplay {
                moves: Vec::new(),
                metadata: metadata_factory(i)
            })
            .collect();

        let result = loop {
            match game.step() {
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
        create_standard_game(|index| {
            let moves = self.snakes[index].moves.clone();
            Box::new(ReplayRobot::new(moves))
        })
    }

    pub fn winner(&self) -> Option<&SnakeReplay<M>> {
        self.result.winner().and_then(|i| self.snakes.get(i))
    }
}
