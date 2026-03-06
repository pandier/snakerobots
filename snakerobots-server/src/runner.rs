use rand::RngExt;
use snakerobots_shared::{Direction, Point, Size, dto, logic::{self, robot::impls::PathfindRobot}};
use tokio::task::JoinHandle;

pub fn run_game() -> JoinHandle<dto::Game> {
    tokio::task::spawn_blocking(run_game_blocking)
}

pub fn run_game_blocking() -> dto::Game {
    let seed = rand::rng().random();

    let width = 25;
    let height = 13;

    let players: Vec<logic::Player> = vec![
        (Point::new(1, height / 2), Direction::Right),
        (Point::new(width - 2, height / 2), Direction::Left),
    ]
    .into_iter()
    .map(|(p, d)| {
        let mut snake = logic::Snake::new(p, d);
        snake.expand_head(d);
        logic::Player::new(snake, Box::new(PathfindRobot::new()))
    })
    .collect();

    let mut snakes: Vec<dto::GameSnake> = players
        .iter()
        .map(|_| dto::GameSnake { moves: Vec::new() })
        .collect();

    let mut game = logic::Game::new(Size::new(width, height), 1, seed, players)
        .expect("predefined layout should be correct");

    let result = loop {
        match game.step() {
            logic::GameStep::Success { moves, added_apples: _, removed_apples: _ } => {
                for (i, dir) in moves {
                    if let Some(snake) = snakes.get_mut(i) {
                        snake.moves.push(dir);
                    }
                }
            }
            logic::GameStep::Finished(result) => break result,
        }
    };

    dto::Game {
        seed,
        snakes,
        result,
    }
}
