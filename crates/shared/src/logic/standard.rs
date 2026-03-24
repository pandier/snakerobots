use rand::RngExt;

use crate::{Direction, Point, Size, logic::{Game, Player, Robot, Snake}};

pub const STANDARD_WIDTH: i32 = 18;
pub const STANDARD_HEIGHT: i32 = 11;
pub const STANDARD_APPLE_COUNT: usize = 1;
pub const MINIMUM_WIDTH: i32 = 7;
pub const MINIMUM_HEIGHT: i32 = 3;

pub fn create_standard_game<F>(robot_factory: F) -> Game
where
    F: Fn(usize) -> Box<dyn Robot>,
{
    create_standard_game_with_size(robot_factory, STANDARD_WIDTH, STANDARD_HEIGHT)
        .expect("standard game should be correct")
}

pub fn create_standard_game_with_size<F>(
    robot_factory: F,
    width: i32,
    height: i32
) -> Result<Game, ()>
where
    F: Fn(usize) -> Box<dyn Robot>,
{
    if width < MINIMUM_WIDTH || height < MINIMUM_HEIGHT {
        return Err(());
    }

    let snakes = vec![
        Snake::new(Point::new(1, height / 2), Direction::Right),
        Snake::new(Point::new(width - 2, height / 2), Direction::Left),
    ];

    let players = snakes
        .into_iter()
        .enumerate()
        .map(|(index, mut snake)| {
            snake.expand_head(snake.direction);
            let robot = robot_factory(index);
            Player::new(snake, robot)
        })
        .collect();

    let seed = rand::rng().random();

    Game::new(Size::new(width, height), STANDARD_APPLE_COUNT, seed, players).ok_or(())
}
