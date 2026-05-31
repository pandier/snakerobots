use std::{thread::sleep, time::{Duration, Instant}};
use rand::RngExt;
use snakerobots_shared::{Point, Size, logic::{
    Grid, GridCell, robot::{error::PropagatingRobotErrorHandler, impls::PathfindRobot, lang::{DEFAULT_HEAP_SIZE, DEFAULT_MAX_INSTRUCTION_COST, DEFAULT_STACK_SIZE, LangRobot}}, standard::create_standard_game
}};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seed = rand::rng().random();

    println!("Seed: {}", seed);

    let lang_robot = LangRobot::compile(r#"
        fn step(game: Game) -> Direction {
            let head = game.snake.points.get(0);

            if (head.y == 0) {
                if (head.x == 0) {
                    return Direction.RIGHT;
                } else if (head.x == 1) {
                    return Direction.DOWN;
                }
            } else if (head.y == 1) {
                if (head.x == 0) {
                    return Direction.UP;
                } else if (head.x == 1) {
                    return Direction.LEFT;
                }
            }

            if (head.y > 1) {
                return Direction.UP;
            }
            return Direction.LEFT;
        }
    "#.to_owned(), DEFAULT_STACK_SIZE, DEFAULT_HEAP_SIZE, DEFAULT_MAX_INSTRUCTION_COST)?;

    let mut game = create_standard_game(Box::new(lang_robot), Some(Box::new(PathfindRobot::new())), Some(seed));

    print_grid(game.grid(), false);

    let mut instant = Instant::now();

    while game.result().is_none() {
        sleep(Duration::from_millis(20).saturating_sub(Instant::now().duration_since(instant)));
        instant = Instant::now();

        game.step::<PropagatingRobotErrorHandler>()?;

        print_grid(game.grid(), true);
    }

    Ok(())
}
