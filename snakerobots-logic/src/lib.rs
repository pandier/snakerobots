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

    pub fn from_snakes<'a>(
        size: Size,
        snakes: impl Iterator<Item = (usize, &'a Snake)>,
    ) -> Option<Self> {
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
        self.cells
            .iter()
            .enumerate()
            .map(|(i, cell)| (self.from_index(i), cell))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Win(usize),
    Tie,
}

pub enum GameStep {
    Success {
        moves: Vec<(usize, Direction)>,
    },
    Finished(GameResult),
}

pub struct Game {
    size: Size,
    players: Vec<Player>,
    apples: HashSet<Point>,
    grid: Grid,
    max_steps_without_apple: usize,
    steps_without_apple: usize,
    result: Option<GameResult>,
    rng: Xoshiro128PlusPlus,
}

impl Game {
    pub fn new(size: Size, apple_count: usize, seed: u64, players: Vec<Player>) -> Option<Self> {
        let grid = Grid::from_snakes(
            size,
            players
                .iter()
                .enumerate()
                .filter_map(|(i, player)| player.snake.as_ref().map(|snake| (i, snake))),
        )
        .unwrap();

        let mut game = Self {
            size,
            players,
            grid,
            apples: HashSet::new(),
            max_steps_without_apple: size.area() as usize,
            steps_without_apple: 0,
            result: None,
            rng: Xoshiro128PlusPlus::seed_from_u64(seed),
        };

        for _ in 0..apple_count {
            game.place_random_apple();
        }

        Some(game)
    }

    pub fn step(&mut self) -> GameStep {
        if let Some(result) = self.result {
            return GameStep::Finished(result);
        }

        let moves = self
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

        for (i, dir) in &moves {
            let player = &mut self.players[*i];

            let Some(snake) = &mut player.snake else {
                continue;
            };

            let new_head = snake.head().direction(*dir);

            if self.size.contains(&new_head) {
                let grow = self.apples.remove(&new_head);
                if grow {
                    eaten_apples += 1;
                    if snake.expand_head(*dir) {
                        continue;
                    }
                } else if snake.advance(*dir) {
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

        if eaten_apples > 0 {
            self.steps_without_apple = 0;
        } else {
            self.steps_without_apple += 1;
        }

        self.result = self.calculate_result();

        GameStep::Success { moves }
    }

    fn iter_snakes(&self) -> impl Iterator<Item = (usize, &Player, &Snake)> {
        self.players
            .iter()
            .enumerate()
            .filter_map(|(i, player)| player.snake.as_ref().map(|snake| (i, player, snake)))
    }

    fn update_grid(&mut self) {
        let snakes = self.iter_snakes().map(|(i, _, snake)| (i, snake));
        let mut new_grid = Grid::from_snakes(self.size, snakes).unwrap();
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

    fn calculate_result(&self) -> Option<GameResult> {
        if self.steps_without_apple >= self.max_steps_without_apple {
            return Some(GameResult::Tie);
        }

        let mut iter = self.iter_snakes();
        match iter.next() {
            Some((i, _, _)) => match iter.next() {
                Some(_) => None,
                None => Some(GameResult::Win(i)),
            }
            None => Some(GameResult::Tie),
        }
    }

    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }
}
