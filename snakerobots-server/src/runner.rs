use rand::RngExt;
use snakerobots_dto::{GameDto, GameMoveDto, GameResultDto, GameSnakeDto};
use snakerobots_logic::{
    Direction, Game, GameResult, GameStep, Player, Point, Size, Snake, robot::impls::PathfindRobot
};
use tokio::task::JoinHandle;

pub fn run_game() -> JoinHandle<GameDto> {
    tokio::task::spawn_blocking(run_game_blocking)
}

pub fn run_game_blocking() -> GameDto {
    let seed = rand::rng().random();

    let width = 25;
    let height = 13;

    let players: Vec<Player> = vec![
        (Point::new(2, height / 2), Direction::Left),
        (Point::new(width - 3, height / 2), Direction::Right),
    ]
    .into_iter()
    .map(|(p, d)| {
        let mut snake = Snake::new(p);
        snake.expand_tail(d);
        Player::new(snake, Box::new(PathfindRobot::new()))
    })
    .collect();

    let mut snakes: Vec<GameSnakeDto> = players
        .iter()
        .map(|_| GameSnakeDto { moves: Vec::new() })
        .collect();

    let mut game = Game::new(Size::new(width, height), 1, seed, players)
        .expect("predefined layout should be correct");

    let result = loop {
        match game.step() {
            GameStep::Success { moves } => {
                for (i, dir) in moves {
                    if let Some(snake) = snakes.get_mut(i) {
                        snake.moves.push(match dir {
                            Direction::Up => GameMoveDto::Up,
                            Direction::Down => GameMoveDto::Down,
                            Direction::Left => GameMoveDto::Left,
                            Direction::Right => GameMoveDto::Right,
                        });
                    }
                }
            }
            GameStep::Finished(result) => break match result {
                GameResult::Win(i) => GameResultDto::Win { winner: i },
                GameResult::Tie => GameResultDto::Tie,
            },
        }
    };

    GameDto {
        seed,
        snakes,
        result,
    }
}
