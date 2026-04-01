use rand::RngExt;

use crate::{Direction, Point, Size, logic::{Game, Player, Robot, Snake}};

pub const STANDARD_WIDTH: i32 = 18;
pub const STANDARD_HEIGHT: i32 = 11;
pub const STANDARD_APPLE_COUNT: usize = 1;
pub const MINIMUM_WIDTH: i32 = 7;
pub const MINIMUM_HEIGHT: i32 = 3;

pub fn create_standard_game(robot1: Box<dyn Robot>, robot2: Box<dyn Robot>) -> Game {
    create_standard_game_with_size(robot1, robot2, STANDARD_WIDTH, STANDARD_HEIGHT)
        .expect("standard game should be correct")
}

pub fn create_standard_game_with_size(
    robot1: Box<dyn Robot>,
    robot2: Box<dyn Robot>,
    width: i32,
    height: i32
) -> Result<Game, ()> {
    if width < MINIMUM_WIDTH || height < MINIMUM_HEIGHT {
        return Err(());
    }

    let players = vec![
        {
            let mut snake = Snake::new(Point::new(1, height / 2), Direction::Right);
            snake.expand_head(snake.direction);
            Player::new(snake, robot1)
        },
        {
            let mut snake = Snake::new(Point::new(width - 2, height / 2), Direction::Left);
            snake.expand_head(snake.direction);
            Player::new(snake, robot2)
        },
    ];

    let seed = rand::rng().random();

    Game::new(Size::new(width, height), STANDARD_APPLE_COUNT, seed, players).ok_or(())
}
