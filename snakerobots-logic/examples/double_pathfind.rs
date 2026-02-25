use std::{collections::HashSet, sync::Mutex, thread::sleep, time::Duration, vec};

use rand::RngExt;
use snakerobots_logic::{
    Direction, Game, GameState, Player, Point, Robot, RobotContext, Size, Snake,
};

#[derive(Clone)]
pub struct Path {
    dir: Option<Direction>,
    snake: Snake,
    evolution: usize,
}

impl Path {
    pub fn new(snake: Snake) -> Self {
        Self {
            dir: None,
            snake,
            evolution: 0,
        }
    }

    pub fn advance_all(&mut self, ctx: &RobotContext) -> impl Iterator<Item = Self> {
        Direction::ALL
            .iter()
            .filter_map(|dir| self.advance(*dir, ctx))
    }

    pub fn advance(&mut self, dir: Direction, ctx: &RobotContext) -> Option<Self> {
        let p = self.snake.head().direction(dir);
        if self.is_safe(p, ctx) {
            let mut snake = self.snake.clone();
            if snake.advance(dir) {
                return Some(Self {
                    dir: Some(self.dir.unwrap_or(dir)),
                    snake,
                    evolution: self.evolution + 1,
                });
            }
        }
        None
    }

    fn is_safe(&self, p: Point, ctx: &RobotContext) -> bool {
        p.x >= 0
            && p.y >= 0
            && p.x < (ctx.size.width as i32)
            && p.y < (ctx.size.height as i32)
            && !self.snake.contains(p)
            && ctx
                .opponents
                .iter()
                .all(|s| Self::is_safe_for_opponent(p, s))
    }

    fn is_safe_for_opponent(p: Point, opponent: &Snake) -> bool {
        if opponent.contains(p) {
            return false;
        }

        let head = opponent.head();
        Direction::ALL
            .into_iter()
            .map(|dir| head.direction(dir))
            .all(|bad| p != bad)
    }
}

pub struct ExampleRobot {
    last: Mutex<Direction>,
}

impl ExampleRobot {
    pub fn new(dir: Direction) -> Self {
        Self {
            last: Mutex::new(dir),
        }
    }

    fn find_path(&self, origin: Path, ctx: &RobotContext, apple: bool) -> Path {
        let mut explored = HashSet::new();
        let mut paths = vec![origin.clone()];

        let mut highest_evolution: usize = 0;
        let mut highest_evolution_path: Path = origin;

        while let Some(mut path) = paths.pop() {
            if !explored.insert(path.snake.head()) {
                continue;
            }

            if apple && ctx.apples.contains(&path.snake.head()) {
                let evolution = path.evolution;
                let final_path = self.find_path(path.clone(), ctx, false);
                if final_path.evolution - evolution > 5 {
                    return final_path;
                }
            }

            if path.evolution > highest_evolution {
                highest_evolution = path.evolution;
                highest_evolution_path = path.clone();
            }

            for new_path in path.advance_all(&ctx) {
                paths.insert(0, new_path);
            }
        }

        highest_evolution_path
    }
}

impl Robot for ExampleRobot {
    fn step(&self, ctx: RobotContext) -> Direction {
        let mut last = self.last.lock().unwrap();

        let path = self.find_path(Path::new(ctx.snake.clone()), &ctx, true);
        let dir = path.dir.unwrap_or(*last);

        *last = dir;
        dir
    }
}

fn build_player(x: i32, y: i32, dir: Direction) -> Player {
    let mut snake = Snake::new(Point::new(x, y));
    snake.expand_tail(dir.opposite());
    Player::new(snake, Box::new(ExampleRobot::new(dir)))
}

fn main() {
    loop {
        let seed = rand::rng().random::<[u8; 16]>();

        println!("Seed: {:x?}", seed);

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
        );

        game.print(false);
        while game.state() == GameState::Active {
            sleep(Duration::from_millis(50));
            game.step();
            game.print(true);
        }

        sleep(Duration::from_secs(2));
    }
}
