use std::{collections::HashSet, sync::Mutex, thread::sleep, time::Duration, vec};

use rand::RngExt;
use snakerobots_logic::{
    Direction, Game, GameState, Player, Point, Robot, RobotContext, Size, Snake,
};

pub struct ExampleRobot {
    last: Mutex<Direction>,
}

impl ExampleRobot {
    pub fn new(dir: Direction) -> Self {
        Self {
            last: Mutex::new(dir),
        }
    }

    fn point(point: Point, dir: Direction, ctx: &RobotContext) -> Option<Point> {
        let p = point.direction(dir);
        if Self::is_safe(p, ctx) { Some(p) } else { None }
    }

    fn is_safe(p: Point, ctx: &RobotContext) -> bool {
        p.x >= 0
            && p.y >= 0
            && p.x < ctx.size.width
            && p.y < ctx.size.height
            && !ctx.snake.contains(p)
            && ctx
                .opponents
                .iter()
                .all(|s| Self::is_safe_for_opponent(p, s))
    }

    fn is_safe_for_opponent(p: Point, opponent: &Snake) -> bool {
        if opponent.contains(p) && opponent.tail() != p {
            return false;
        }

        let head = opponent.head();
        Direction::ALL
            .into_iter()
            .map(|dir| head.direction(dir))
            .all(|bad| p != bad)
    }
}

impl Robot for ExampleRobot {
    fn step(&self, ctx: RobotContext) -> Direction {
        let mut last = self.last.lock().unwrap();
        let head = ctx.snake.head();

        if !ctx.apples.is_empty() {
            let mut explored = HashSet::new();
            explored.insert(head);

            let mut paths = Direction::ALL
                .into_iter()
                .filter_map(|dir| Self::point(head, dir, &ctx).map(|p| (p, dir)))
                .collect::<Vec<_>>();

            while let Some((p, original_dir)) = paths.pop() {
                if !explored.insert(p) {
                    continue;
                }

                *last = original_dir;

                if ctx.apples.contains(&p) {
                    return original_dir;
                }

                for dir in Direction::ALL {
                    if let Some(new_p) = Self::point(p, dir, &ctx) {
                        if !explored.contains(&new_p) {
                            paths.insert(0, (new_p, original_dir));
                        }
                    }
                }
            }
        }

        *last
    }
}

fn build_player(x: i32, y: i32, dir: Direction) -> Player {
    let mut snake = Snake::new(Point::new(x, y));
    snake.expand_tail(dir.opposite());
    Player::new(snake, Box::new(ExampleRobot::new(dir)))
}

fn main() {
    let seed = rand::rng().random::<[u8; 16]>();

    println!("Seed: {:x?}", seed);

    let width = 17;
    let height = 9;
    let mut game = Game::new(
        Size::new(width, height),
        2,
        seed,
        vec![
            build_player(2, 2, Direction::Right),
            build_player(width - 3, 2, Direction::Left),
            build_player(width / 2, height - 3, Direction::Up),
        ],
    );

    game.print(false);
    while game.state() == GameState::Active {
        sleep(Duration::from_millis(200));
        game.step();
        game.print(true);
    }
}
