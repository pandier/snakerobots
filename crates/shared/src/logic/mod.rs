pub mod robot;
pub mod standard;

use std::{cell::RefCell, collections::HashSet, fmt::Debug};

use rand::{RngExt, SeedableRng, rngs::Xoshiro128PlusPlus};

pub use robot::{Robot, RobotContext};

use crate::{Direction, GameResult, Point, Size, logic::robot::error::{InfallibleRobotErrorHandler, RobotErrorHandler}};

#[derive(Debug, Clone)]
pub struct Snake {
    points: Vec<Point>,
    direction: Direction,
}

impl Snake {
    pub fn new(point: Point, direction: Direction) -> Self {
        Self {
            points: vec![point],
            direction,
        }
    }

    pub fn head(&self) -> Point {
        *self.points.first().expect("snake should not be empty")
    }

    pub fn tail(&self) -> Point {
        *self.points.last().expect("snake should not be empty")
    }

    pub fn points(&self) -> &Vec<Point> {
        &self.points
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn contains(&self, point: Point) -> bool {
        self.points.contains(&point)
    }

    pub fn advance(&mut self, dir: Direction) -> bool {
        if dir == self.direction.opposite() {
            return false;
        }

        let new_head = self.head().direction(dir);
        if new_head != self.tail() && self.contains(new_head) {
            false
        } else {
            self.pop_tail();
            self.push_head(new_head);
            self.direction = dir;
            true
        }
    }

    pub fn expand_head(&mut self, dir: Direction) -> bool {
        if dir == self.direction.opposite() {
            return false;
        }

        let new_head = self.head().direction(dir);
        if self.contains(new_head) {
            false
        } else {
            self.push_head(new_head);
            self.direction = dir;
            true
        }
    }

    pub fn expand_tail(&mut self, dir: Direction) -> bool {
        let new_tail = self.tail().direction(dir);
        if self.contains(new_tail) {
            false
        } else {
            self.push_tail(new_tail);
            true
        }
    }

    pub fn push_head(&mut self, point: Point) {
        self.points.insert(0, point);
    }

    pub fn push_tail(&mut self, point: Point) {
        self.points.push(point);
    }

    pub fn pop_head(&mut self) -> Option<Point> {
        if self.points.len() > 1 {
            Some(self.points.remove(0))
        } else {
            None
        }
    }

    pub fn pop_tail(&mut self) -> Option<Point> {
        if self.points.len() > 1 {
            self.points.pop()
        } else {
            None
        }
    }
}

#[cfg(feature = "lang")]
impl crate::lang::util::arg_convertor::ValueLike for Snake {
    fn into_convertable(self) -> crate::lang::util::arg_convertor::ValueConvertable {
        let mut s = crate::lang::util::arg_convertor::struct_convertor("Snake");
        s.add_field(self.points);
        s.add_field(self.direction);
        s.convert()
    }
}

pub struct Player {
    snake: Option<Snake>,
    robot: RefCell<Box<dyn Robot>>,
}

impl Player {
    pub fn new(snake: Snake, robot: Box<dyn Robot>) -> Self {
        Self {
            snake: Some(snake),
            robot: RefCell::new(robot),
        }
    }

    pub fn snake(&self) -> Option<&Snake> {
        self.snake.as_ref()
    }

    pub fn is_alive(&self) -> bool {
        self.snake.is_some()
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

pub enum GameStep {
    Success {
        moves: Vec<(usize, Direction)>,
        added_apples: Vec<Point>,
        removed_apples: Vec<Point>,
    },
    Finished(GameResult),
}

pub struct Game {
    size: Size,
    seed: u64,
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
            seed,
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

    pub fn step_infallible(&mut self) -> GameStep {
        self.step::<InfallibleRobotErrorHandler>().expect("infallible")
    }

    pub fn step<E: RobotErrorHandler>(&mut self) -> Result<GameStep, E::Error> {
        if let Some(result) = self.result.clone() {
            return Ok(GameStep::Finished(result));
        }

        let ctxs = self
            .iter_snakes()
            .map(|(i, _, snake)| {
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
                (i, ctx)
            })
            .collect::<Vec<_>>();

        let mut moves = Vec::new();

        for (i, ctx) in ctxs {
            let player = &mut self.players[i];
            let mut robot = player.robot.borrow_mut();
            let result = robot.step(ctx);
            match result {
                Ok(dir) => {
                    moves.push((i, dir));
                },
                Err(err) => {
                    E::handle(err)?;
                    player.snake = None;
                }
            }
        }

        let mut added_apples = Vec::new();
        let mut removed_apples = Vec::new();

        for (i, dir) in &moves {
            let player = &mut self.players[*i];

            let Some(snake) = &mut player.snake else {
                continue;
            };

            let new_head = snake.head().direction(*dir);

            if self.size.contains(&new_head) {
                let grow = self.apples.remove(&new_head);
                if grow {
                    removed_apples.push(new_head);

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

        for _ in 0..removed_apples.len() {
            if let Some(apple) = self.place_random_apple() {
                added_apples.push(apple);
            }
        }

        if removed_apples.len() > 0 {
            self.steps_without_apple = 0;
        } else {
            self.steps_without_apple += 1;
        }

        self.result = self.calculate_result();

        Ok(GameStep::Success {
            moves,
            added_apples,
            removed_apples,
        })
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

    fn place_random_apple(&mut self) -> Option<Point> {
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
            Some(point)
        } else {
            None
        }
    }

    fn calculate_result(&self) -> Option<GameResult> {
        if self.steps_without_apple >= self.max_steps_without_apple {
            return Some(GameResult::Tie);
        }

        let mut iter = self.iter_snakes();
        match iter.next() {
            Some((winner, _, _)) => match iter.next() {
                Some(_) => None,
                None => Some(GameResult::Win { winner }),
            }
            None => Some(GameResult::Tie),
        }
    }

    pub fn result(&self) -> &Option<GameResult> {
        &self.result
    }

    pub fn snakes(&self) -> Vec<&Snake> {
        self.iter_snakes()
            .map(|(_, _, s)| s)
            .collect()
    }

    pub fn players(&self) -> &Vec<Player> {
        &self.players
    }

    pub fn apples(&self) -> &HashSet<Point> {
        &self.apples
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn size(&self) -> Size {
        self.size
    }
}
