use rand::RngExt;

use crate::{Direction, Point, Size, logic::{Game, Player, Snake, robot::impls::PathfindRobot}};

pub const STANDARD_WIDTH: i32 = 18;
pub const STANDARD_HEIGHT: i32 = 11;
pub const STANDARD_APPLE_COUNT: usize = 1;
pub const MINIMUM_WIDTH: i32 = 7;
pub const MINIMUM_HEIGHT: i32 = 3;

pub fn create_standard_game() -> Game {
    create_standard_game_with_size(STANDARD_WIDTH, STANDARD_HEIGHT)
        .expect("standard game should be correct")
}

pub fn create_standard_game_with_size(width: i32, height: i32) -> Result<Game, ()> {
    let players = create_standard_snakes_for_size(width, height)?
        .into_iter()
        .map(|snake| Player::new(snake, Box::new(PathfindRobot::new())))
        .collect();

    let seed = rand::rng().random();

    Game::new(Size::new(width, height), STANDARD_APPLE_COUNT, seed, players).ok_or(())
}

pub fn create_standard_snakes() -> Vec<Snake> {
    create_standard_snakes_for_size(STANDARD_WIDTH, STANDARD_HEIGHT)
        .expect("standard snakes should be correct")
}

pub fn create_standard_snakes_for_size(width: i32, height: i32) -> Result<Vec<Snake>, ()> {
    if width < MINIMUM_WIDTH || height < MINIMUM_HEIGHT {
        return Err(());
    }

    let mut snakes = vec![
        Snake::new(Point::new(1, height / 2), Direction::Right),
        Snake::new(Point::new(width - 2, height / 2), Direction::Left),
    ];
    for snake in &mut snakes {
        snake.expand_head(snake.direction);
    }
    Ok(snakes)
}
