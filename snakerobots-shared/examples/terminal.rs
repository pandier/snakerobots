use std::{
    thread::sleep,
    time::{Duration, Instant},
    vec,
};

use rand::RngExt;
use snakerobots_shared::{Direction, Point, Size, logic::{Game, Grid, GridCell, Player, Snake, robot::impls::PathfindRobot}};

fn build_player(x: i32, y: i32, dir: Direction) -> Player {
    let mut snake = Snake::new(Point::new(x, y));
    snake.expand_tail(dir.opposite());
    Player::new(snake, Box::new(PathfindRobot::new()))
}

fn fmt_cell(space: &GridCell) -> String {
    match space {
        GridCell::Empty => String::from("  "),
        GridCell::Apple => String::from("\x1B[30;41m  \x1B[0m"),
        GridCell::Snake(i) => format!("\x1B[30;{0}m  \x1B[0m", 42 + i),
    }
}

fn print_grid(grid: &Grid, clear: bool) {
    let Size { width, height } = grid.size();
    let mut output = String::new();

    if clear {
        output += &format!("\x1B[{}F", height + 2);
    }

    output += &format!("+{}+\n", "--".repeat(width as usize));
    for y in 0..height {
        output += &format!(
            "|{}|\n",
            (0..width)
                .filter_map(|x| grid.get(&Point::new(x, y)))
                .map(|c| fmt_cell(c))
                .collect::<String>()
        );
    }
    output += &format!("+{}+\n", "--".repeat(width as usize));
    print!("{}", output);
}

fn main() {
    let seed = rand::rng().random();

    println!("Seed: {}", seed);

    let width = 25;
    let height = 13;
    let mut game = Game::new(
        Size::new(width, height),
        2,
        seed,
        vec![
            build_player(2, 2, Direction::Right),
            build_player(width - 3, 2, Direction::Left),
            build_player(2, height - 3, Direction::Right),
            build_player(width - 3, height - 3, Direction::Left),
            build_player(width / 2, height - 3, Direction::Up),
            build_player(width / 2, 2, Direction::Down),
        ],
    )
    .expect("invalid game layout");

    print_grid(game.grid(), false);

    let mut instant = Instant::now();

    while game.result().is_none() {
        sleep(Duration::from_millis(50).saturating_sub(Instant::now().duration_since(instant)));
        instant = Instant::now();

        game.step();

        print_grid(game.grid(), true);
    }
}
