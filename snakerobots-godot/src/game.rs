use std::{collections::{HashMap, HashSet}, i64};

use godot::prelude::*;
use rand::RngExt;
use snakerobots_shared::{Direction, Point, Size, logic::{Game, GameStep, Player, Snake, robot::impls::PathfindRobot}};

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct GameSnake {
    #[var]
    points: Array<Vector2i>,
}

#[godot_api]
impl GameSnake {
    #[func]
    pub fn create(points: Array<Vector2i>) -> Gd<Self> {
        Gd::from_object(GameSnake { points })
    }

    #[func]
    pub fn points(&self) -> Array<Vector2i> {
        self.points.clone()
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct GameTimeline {
    length: usize,
    width: i32,
    height: i32,
    snakes: Vec<GameTimelineSnake>,
    apples: GameTimelineApples,
    time: usize,
}

#[godot_api]
impl GameTimeline {
    #[func]
    pub fn create(width: i32, height: i32) -> Option<Gd<GameTimeline>> {
        let seed = rand::rng().random();

        if width < 7 || height < 3 {
            return None;
        }

        let players: Vec<Player> = vec![
            (Point::new(1, height / 2), Direction::Right),
            (Point::new(width - 2, height / 2), Direction::Left),
        ]
        .into_iter()
        .map(|(p, d)| {
            let mut snake = Snake::new(p, d);
            snake.expand_head(d);
            Player::new(snake, Box::new(PathfindRobot::new()))
        })
        .collect();

        let game = Game::new(Size::new(width, height), 1, seed, players)
            .expect("predefined layout should be correct");

        Some(Gd::from_object(Self::run_game(game)))
    }

    pub fn run_game(mut game: Game) -> GameTimeline {
        let mut builder = GameTimelineBuilder::new(game.snakes(), game.apples().clone());
        loop {
            match game.step() {
                GameStep::Success { moves, added_apples, removed_apples } => {
                    for (i, _) in moves {
                        if let Some(snake) = game.players().get(i).and_then(|p| p.snake()) {
                            builder.record_snake(i, snake);
                        }
                    }
                    builder.record_apples(added_apples, removed_apples);
                }
                GameStep::Finished(_) => break,
            }
        };
        builder.build(game.size(), game.apples().clone())
    }

    #[func]
    pub fn forward(&mut self) -> bool {
        if self.time >= self.length {
            return false;
        }

        for snake in &mut self.snakes {
            snake.forward(self.time);
        }
        self.apples.forward(self.time);
        self.time += 1;

        return true;
    }

    #[func]
    pub fn backward(&mut self) -> bool {
        if self.time <= 0 {
            return false;
        }

        self.time -= 1;
        for snake in &mut self.snakes {
            snake.backward(self.time);
        }
        self.apples.backward(self.time);

        return true;
    }

    #[func]
    pub fn goto_start(&mut self) {
        self.time = 0;
        for snake in &mut self.snakes {
            snake.goto_start();
        }
        self.apples.goto_start();
    }

    #[func]
    pub fn goto_end(&mut self) {
        self.time = self.length;
        for snake in &mut self.snakes {
            snake.goto_end();
        }
        self.apples.goto_end();
    }

    #[func]
    pub fn skip_to(&mut self, time: i64) {
        let target = usize::try_from(time).unwrap_or(0).clamp(0, self.length);

        let dst_to_time = self.time.abs_diff(target);
        let dst_to_end = self.length - target;

        if target < dst_to_time {
            self.goto_start();
        } else if dst_to_end < dst_to_time {
            self.goto_end();
        }

        if target > self.time {
            while target > self.time && self.forward() {}
        } else if target < self.time {
            while target < self.time && self.backward() {}
        }
    }

    #[func]
    pub fn get_snakes(&self) -> Array<Gd<GameSnake>> {
        Self::convert_snakes(self.time, &self.snakes)
    }

    #[func]
    pub fn get_next_snakes(&self) -> Array<Gd<GameSnake>> {
        if self.time >= self.length {
            return Array::new();
        }

        let mut next_snakes = self.snakes.clone();
        for snake in &mut next_snakes {
            snake.forward(self.time);
        }
        Self::convert_snakes(self.time + 1, &next_snakes)
    }

    fn convert_snakes<'a>(time: usize, snakes: &Vec<GameTimelineSnake>) -> Array<Gd<GameSnake>> {
        snakes.iter()
            .filter(|snake| snake.is_alive(time))
            .map(|snake| GameSnake::create(snake.current.points().iter()
                .map(|point| Vector2i::new(point.x, point.y))
                .collect::<Array<_>>()))
            .collect::<Array<_>>()
    }

    #[func]
    pub fn get_apples(&self) -> Array<Vector2i> {
        self.apples.current.iter()
            .map(|point| Vector2i::new(point.x, point.y))
            .collect::<Array<_>>()
    }

    #[func]
    pub fn get_time(&self) -> u64 {
        self.time as u64
    }

    #[func]
    pub fn get_length(&self) -> u64 {
        self.length as u64
    }

    #[func]
    pub fn get_width(&self) -> i32 {
        self.width
    }

    #[func]
    pub fn get_height(&self) -> i32 {
        self.height
    }
}

#[derive(Debug, Clone)]
pub struct GameTimelineSnakeStep {
    tail: Option<Point>,
    head: Point,
}

#[derive(Debug, Clone)]
pub struct GameTimelineSnake {
    start: Snake,
    end: Snake,
    steps: Vec<GameTimelineSnakeStep>,
    current: Snake,
}

impl GameTimelineSnake {
    pub fn forward(&mut self, time: usize) {
        if let Some(step) = self.steps.get(time) {
            self.current.push_head(step.head);
            if step.tail.is_some() {
                self.current.pop_tail();
            }
        }
    }

    pub fn backward(&mut self, time: usize) {
        if let Some(step) = self.steps.get(time) {
            if let Some(tail) = step.tail {
                self.current.push_tail(tail);
            }
            self.current.pop_head();
        }
    }

    pub fn goto_start(&mut self) {
        self.current = self.start.clone();
    }

    pub fn goto_end(&mut self) {
        self.current = self.end.clone();
    }

    pub fn is_alive(&self, time: usize) -> bool {
        time <= self.steps.len()
    }
}

pub struct GameTimelineAppleStep {
    removed: Vec<Point>,
    added: Vec<Point>,
}

pub struct GameTimelineApples {
    start: HashSet<Point>,
    end: HashSet<Point>,
    steps: HashMap<usize, GameTimelineAppleStep>,
    current: HashSet<Point>,
}

impl GameTimelineApples {
    pub fn forward(&mut self, time: usize) {
        if let Some(step) = self.steps.get(&time) {
            for apple in &step.removed {
                self.current.remove(apple);
            }
            for apple in &step.added {
                self.current.insert(*apple);
            }
        }
    }

    pub fn backward(&mut self, time: usize) {
        if let Some(step) = self.steps.get(&time) {
            for apple in &step.added {
                self.current.remove(apple);
            }
            for apple in &step.removed {
                self.current.insert(*apple);
            }
        }
    }

    pub fn goto_start(&mut self) {
        self.current = self.start.clone();
    }

    pub fn goto_end(&mut self) {
        self.current = self.end.clone();
    }
}

pub struct GameTimelineBuilder {
    snakes: Vec<GameTimelineSnakeBuilder>,
    start_apples: HashSet<Point>,
    apples: HashMap<usize, GameTimelineAppleStep>,
    index: usize,
}

impl GameTimelineBuilder {
    pub fn new(snakes: Vec<&Snake>, apples: HashSet<Point>) -> Self {
        let snakes = snakes
            .into_iter()
            .map(|snake| GameTimelineSnakeBuilder::new(snake.clone()))
            .collect();
        Self {
            snakes,
            start_apples: apples.clone(),
            apples: HashMap::new(),
            index: 0,
        }
    }

    pub fn record_snake(&mut self, time: usize, snake: &Snake) {
        if let Some(builder) = self.snakes.get_mut(time) {
            builder.record(snake);
        }
    }

    pub fn record_apples(&mut self, added: Vec<Point>, removed: Vec<Point>) {
        if !added.is_empty() || !removed.is_empty() {
            self.apples.insert(self.index, GameTimelineAppleStep { added, removed });
        }

        self.index += 1;
    }

    pub fn build(self, size: Size, apples: HashSet<Point>) -> GameTimeline {
        GameTimeline {
            length: self.index,
            width: size.width,
            height: size.height,
            snakes: self.snakes.into_iter().map(|x| x.build()).collect(),
            apples: GameTimelineApples {
                start: self.start_apples.clone(),
                end: apples,
                steps: self.apples,
                current: self.start_apples,
            },
            time: 0
        }
    }
}

pub struct GameTimelineSnakeBuilder {
    start: Snake,
    current: Snake,
    steps: Vec<GameTimelineSnakeStep>,
}

impl GameTimelineSnakeBuilder {
    pub fn new(snake: Snake) -> Self {
        Self {
            start: snake.clone(),
            current: snake,
            steps: Vec::new(),
        }
    }

    pub fn record(&mut self, snake: &Snake) {
        let tail: Point = self.current.tail();
        let step = if tail == snake.tail() {
            GameTimelineSnakeStep {
                tail: None,
                head: snake.head(),
            }
        } else {
            GameTimelineSnakeStep {
                head: snake.head(),
                tail: Some(tail),
            }
        };
        self.steps.push(step);
        self.current = snake.clone();
    }

    pub fn build(self) -> GameTimelineSnake {
        GameTimelineSnake {
            start: self.start.clone(),
            end: self.current,
            steps: self.steps,
            current: self.start,
        }
    }
}
