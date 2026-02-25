pub mod math;
pub mod robot;

use std::{collections::HashSet, fmt::Debug};

use rand::{RngExt, SeedableRng, rngs::Xoshiro128PlusPlus};

pub use math::{Direction, Point, Size};
pub use robot::{Robot, RobotContext};

#[derive(Debug, Clone)]
pub struct Snake(Vec<Point>);

impl Snake {
    pub fn new(point: Point) -> Self {
        Self(vec![point])
    }

    pub fn head(&self) -> Point {
        *self.0.first().expect("snake should not be empty")
    }

    pub fn tail(&self) -> Point {
        *self.0.last().expect("snake should not be empty")
    }

    pub fn points(&self) -> &Vec<Point> {
        &self.0
    }

    pub fn contains(&self, point: Point) -> bool {
        self.0.contains(&point)
    }

    pub fn advance(&mut self, dir: Direction) -> bool {
        let new_head = self.head().direction(dir);
        if new_head != self.tail() && self.contains(new_head) {
            false
        } else {
            self.0.pop();
            self.0.insert(0, new_head);
            true
        }
    }

    pub fn expand_head(&mut self, dir: Direction) -> bool {
        let new_head = self.head().direction(dir);
        if self.contains(new_head) {
            false
        } else {
            self.0.insert(0, new_head);
            true
        }
    }

    pub fn expand_tail(&mut self, dir: Direction) -> bool {
        let new_tail = self.tail().direction(dir);
        if self.contains(new_tail) {
            false
        } else {
            self.0.push(new_tail);
            true
        }
    }
}

pub struct Player {
    snake: Option<Snake>,
    robot: Box<dyn Robot>,
}

impl Player {
    pub fn new(snake: Snake, robot: Box<dyn Robot>) -> Self {
        Self {
            snake: Some(snake),
            robot,
        }
    }
}

pub struct Game {
    size: Size,
    players: Vec<Player>,
    apples: HashSet<Point>,
    rng: Xoshiro128PlusPlus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Active,
    Win(usize),
    Tie,
}

impl Game {
    pub fn new(size: Size, apple_count: usize, seed: [u8; 16], players: Vec<Player>) -> Self {
        let mut game = Self {
            size,
            players,
            apples: HashSet::new(),
            rng: Xoshiro128PlusPlus::from_seed(seed),
        };
        for _ in 0..apple_count {
            game.place_random_apple();
        }
        game
    }

    fn iter_snakes(&self) -> impl Iterator<Item = (usize, &Player, &Snake)> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, player)| player.snake.as_ref().map(|snake| (i, player, snake)))
    }

    fn iter_points(&self) -> impl Iterator<Item = Point> {
        (0..self.size.height)
            .flat_map(|y| (0..self.size.width).map(move |x| Point::new(x as i32, y as i32)))
    }

    fn place_random_apple(&mut self) {
        let points = self
            .iter_points()
            .filter(|p| {
                !self.iter_snakes().any(|(_, _, s)| s.contains(*p))
                    && !self.apples.iter().any(|a| *a == *p)
            })
            .collect::<Vec<_>>();
        if !points.is_empty() {
            let point = points[self.rng.random_range(..points.len())];
            self.apples.insert(point);
        }
    }

    pub fn step(&mut self) {
        let dirs = self
            .iter_snakes()
            .map(|(i, player, snake)| {
                let ctx = RobotContext {
                    size: self.size,
                    snake: snake.clone(),
                    opponents: self
                        .iter_snakes()
                        .filter(|(j, _, _)| *j != i)
                        .map(|(_, _, s)| s.clone())
                        .collect(),
                    apples: self.apples.clone(),
                };
                let dir = player.robot.step(ctx);
                (i, dir)
            })
            .collect::<Vec<_>>();

        let mut eaten_apples = 0usize;

        for (i, dir) in dirs {
            let player = &mut self.players[i];

            let Some(snake) = &mut player.snake else {
                continue;
            };

            let new_head = snake.head().direction(dir);

            // TODO: is_in_bounds function
            if self.size.contains(&new_head) {
                let grow = self.apples.remove(&new_head);
                if grow {
                    eaten_apples += 1;
                    if snake.expand_head(dir) {
                        continue;
                    }
                } else if snake.advance(dir) {
                    continue;
                }
            }

            player.snake = None;
        }

        let mut collided_players = Vec::new();

        for (i, _, snake) in self.iter_snakes() {
            let head = snake.head();

            for (j, _, other) in self.iter_snakes() {
                if j != i && other.contains(head) {
                    collided_players.push(i);
                    break;
                }
            }
        }

        for i in collided_players {
            let player = &mut self.players[i];
            player.snake = None;
        }

        for _ in 0..eaten_apples {
            self.place_random_apple();
        }
    }

    pub fn state(&self) -> GameState {
        // TODO: Don't iterate twice
        match self.iter_snakes().count() {
            0 => GameState::Tie,
            1 => GameState::Win(self.iter_snakes().next().unwrap().0),
            _ => GameState::Active,
        }
    }

    pub fn print(&self, clear: bool) {
        let width = self.size.width;
        let height = self.size.height;

        let mut grid = Vec::new();
        for _ in 0..(width * height) {
            grid.push(String::from("  "));
        }

        for (i, player) in self.players.iter().enumerate() {
            let Some(snake) = &player.snake else {
                continue;
            };

            let tile = format!("\x1B[30;{0}m  \x1B[0m", 42 + i);

            for point in snake.points() {
                grid[(point.x + point.y * width as i32) as usize] = tile.clone();
            }
        }

        for apple in &self.apples {
            grid[(apple.x + apple.y * width as i32) as usize] = "\x1B[30;41m  \x1B[0m".into();
        }

        let mut output = String::new();

        if clear {
            output += &format!("\x1B[{}F", height + 2);
        }

        output += &format!("+{}+\n", "--".repeat(width as usize));
        for y in 0..height {
            let i = (y * width) as usize;
            output += &format!(
                "|{}|\n",
                grid[i..(i + width as usize)]
                    .iter()
                    .map(|x| x.as_str())
                    .collect::<String>()
            );
        }
        output += &format!("+{}+\n", "--".repeat(width as usize));
        print!("{}", output);
    }
}
