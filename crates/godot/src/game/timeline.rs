use std::{
    collections::{HashMap, HashSet},
    i64,
};

use godot::prelude::*;
use snakerobots_shared::{
    Direction, Point, Size, logic::{Game, GameStep, Snake, robot::error::RobotErrorHandler}
};

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

    pub fn evaluate<E: RobotErrorHandler>(mut game: Game, ids: Vec<Option<String>>) -> Result<GameTimeline, E::Error> {
        let mut builder = GameTimelineBuilder::new(game.snakes(), ids, game.apples().clone());
        loop {
            match game.step::<E>()? {
                GameStep::Success {
                    moves,
                    added_apples,
                    removed_apples,
                } => {
                    for (i, direction) in moves {
                        if let Some(snake) = game.players().get(i).and_then(|p| p.snake()) {
                            builder.record_snake(i, snake, direction);
                        }
                    }
                    builder.record_apples(added_apples, removed_apples);
                }
                GameStep::Finished(_) => break,
            }
        }
        let timeline = builder.build(game.size(), game.apples().clone());
        Ok(timeline)
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
            return self.get_snakes();
        }

        let mut next_snakes = self.snakes.clone();
        for snake in &mut next_snakes {
            snake.forward(self.time);
        }
        Self::convert_snakes(self.time + 1, &next_snakes)
    }

    fn convert_snakes<'a>(time: usize, snakes: &Vec<GameTimelineSnake>) -> Array<Gd<GameSnake>> {
        snakes
            .iter()
            .map(|snake| {
                GameSnake::create(
                    snake
                        .current
                        .points()
                        .iter()
                        .map(|point| Vector2i::new(point.x, point.y))
                        .collect::<Array<_>>(),
                    snake.is_alive(time),
                    snake.get_direction(time),
                    snake.user_id.as_ref().map(|x| x.to_variant()).unwrap_or_else(Variant::nil)
                )
            })
            .collect::<Array<_>>()
    }

    #[func]
    pub fn get_apples(&self) -> Array<Vector2i> {
        self.apples
            .current
            .iter()
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

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct GameSnake {
    #[var]
    pub points: Array<Vector2i>,
    #[var]
    pub is_alive: bool,
    #[var]
    pub direction: GString,
    #[var]
    pub user_id: Variant,
}

#[godot_api]
impl GameSnake {
    pub fn create(points: Array<Vector2i>, is_alive: bool, direction: Option<Direction>, user_id: Variant) -> Gd<Self> {
        Gd::from_object(GameSnake {
            points,
            is_alive,
            direction: direction
                .map(|d| d.to_string().to_godot())
                .unwrap_or_else(|| "None".to_godot()),
            user_id,
        })
    }

    #[func]
    pub fn points(&self) -> Array<Vector2i> {
        self.points.clone()
    }
}

#[derive(Debug, Clone)]
pub struct GameTimelineSnakeStep {
    tail: Option<Point>,
    head: Point,
    direction: Direction,
}

#[derive(Debug, Clone)]
pub struct GameTimelineSnake {
    start: Snake,
    end: Snake,
    steps: Vec<GameTimelineSnakeStep>,
    current: Snake,
    user_id: Option<String>,
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

    pub fn get_direction(&self, time: usize) -> Option<Direction> {
        self.steps.get(time).map(|step| step.direction)
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
    pub fn new(snakes: Vec<&Snake>, ids: Vec<Option<String>>, apples: HashSet<Point>) -> Self {
        let snakes = snakes
            .into_iter()
            .enumerate()
            .map(|(i, snake)| GameTimelineSnakeBuilder::new(snake.clone(), ids[i].clone()))
            .collect();
        Self {
            snakes,
            start_apples: apples.clone(),
            apples: HashMap::new(),
            index: 0,
        }
    }

    pub fn record_snake(&mut self, time: usize, snake: &Snake, direction: Direction) {
        if let Some(builder) = self.snakes.get_mut(time) {
            builder.record(snake, direction);
        }
    }

    pub fn record_apples(&mut self, added: Vec<Point>, removed: Vec<Point>) {
        if !added.is_empty() || !removed.is_empty() {
            self.apples
                .insert(self.index, GameTimelineAppleStep { added, removed });
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
            time: 0,
        }
    }
}

pub struct GameTimelineSnakeBuilder {
    start: Snake,
    current: Snake,
    steps: Vec<GameTimelineSnakeStep>,
    user_id: Option<String>,
}

impl GameTimelineSnakeBuilder {
    pub fn new(snake: Snake, user_id: Option<String>) -> Self {
        Self {
            start: snake.clone(),
            current: snake,
            steps: Vec::new(),
            user_id,
        }
    }

    pub fn record(&mut self, snake: &Snake, direction: Direction) {
        let tail: Point = self.current.tail();
        let step = if tail == snake.tail() {
            GameTimelineSnakeStep {
                tail: None,
                head: snake.head(),
                direction,
            }
        } else {
            GameTimelineSnakeStep {
                head: snake.head(),
                tail: Some(tail),
                direction,
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
            user_id: self.user_id,
        }
    }
}