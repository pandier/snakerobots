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

pub struct Grid {
    size: Size,
    cells: Vec<GridCell>,
}

impl Grid {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cells: vec![GridCell::Empty; size.area() as usize],
        }
    }

    pub fn from_snakes<'a>(size: Size, snakes: impl Iterator<Item = (usize, &'a Snake)>) -> Option<Self> {
        let mut grid = Self::new(size);
        for (i, snake) in snakes {
            for point in snake.points() {
                if let Some(cell) = grid.get_mut(point) {
                    if *cell != GridCell::Empty {
                        return None;
                    }
                    *cell = GridCell::Snake(i);
                }
            }
        }
        Some(grid)
    }

    fn to_index(&self, point: &Point) -> Option<usize> {
        usize::try_from(point.x + self.size.width * point.y).ok()
    }

    fn from_index(&self, index: usize) -> Point {
        let index = index as i32;
        Point::new(index % self.size.width, index / self.size.width)
    }

    pub fn get(&self, point: &Point) -> Option<&GridCell> {
        self.to_index(point).and_then(|i| self.cells.get(i))
    }

    pub fn get_mut(&mut self, point: &Point) -> Option<&mut GridCell> {
        self.to_index(point).and_then(|i| self.cells.get_mut(i))
    }

    pub fn iter(&self) -> impl Iterator<Item = (Point, &GridCell)> {
        self.cells.iter().enumerate().map(|(i, cell)| (self.from_index(i), cell))
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GridCell {
    Empty,
    Snake(usize),
    Apple,
}

pub struct Game {
    size: Size,
    players: Vec<Player>,
    apples: HashSet<Point>,
    grid: Grid,
    rng: Xoshiro128PlusPlus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Active,
    Win(usize),
    Tie,
}

impl Game {
    pub fn new(size: Size, apple_count: usize, seed: [u8; 16], players: Vec<Player>) -> Option<Self> {
        let grid = Grid::from_snakes(size, players.iter().enumerate().filter_map(|(i, player)| player.snake.as_ref().map(|snake| (i, snake)))).unwrap();

        let mut game = Self {
            size,
            players,
            grid,
            apples: HashSet::new(),
            rng: Xoshiro128PlusPlus::from_seed(seed),
        };

        for _ in 0..apple_count {
            game.place_random_apple();
        }

        Some(game)
    }

    fn iter_snakes(&self) -> impl Iterator<Item = (usize, &Player, &Snake)> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, player)| player.snake.as_ref().map(|snake| (i, player, snake)))
    }

    fn update_grid(&mut self) {
        let mut new_grid = Grid::from_snakes(self.size, self.iter_snakes().map(|(i, _, snake)| (i, snake))).unwrap();
        for apple in &self.apples {
            *new_grid.get_mut(apple).unwrap() = GridCell::Apple;
        }
        self.grid = new_grid;
    }

    fn place_random_apple(&mut self) {
        let points = self
            .grid
            .iter()
            .filter(|(_, p)| **p == GridCell::Empty)
            .map(|(p, _)| p)
            .collect::<Vec<_>>();

        if !points.is_empty() {
            let point = points[self.rng.random_range(..points.len())];
            self.apples.insert(point);
            *self.grid.get_mut(&point).unwrap() = GridCell::Apple;
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

        self.update_grid();

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

    pub fn grid(&self) -> &Grid {
        &self.grid
    }
}
