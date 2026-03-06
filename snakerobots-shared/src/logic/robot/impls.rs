use std::{collections::HashSet, sync::Mutex};

use crate::{Direction, Point, logic::{Robot, RobotContext, Snake}};

#[derive(Clone)]
struct Path {
    prev_dir: Direction,
    dir: Option<Direction>,
    point: Point,
    iteration: usize,
}

impl Path {
    pub fn new(point: Point, dir: Direction) -> Self {
        Self {
            dir: None,
            prev_dir: dir,
            point,
            iteration: 0,
        }
    }

    pub fn fork(&self) -> Self {
        Self {
            dir: self.dir.clone(),
            prev_dir: self.prev_dir.clone(),
            point: self.point,
            iteration: 0,
        }
    }

    pub fn advance_all(&mut self, ctx: &RobotContext) -> impl Iterator<Item = Self> {
        Direction::ALL
            .iter()
            .filter_map(|dir| self.advance(*dir, ctx))
    }

    pub fn advance(&mut self, dir: Direction, ctx: &RobotContext) -> Option<Self> {
        if dir != self.prev_dir.opposite() {
            let point = self.point.direction(dir);
            if self.is_safe(point, ctx) {
                return Some(Self {
                    dir: Some(self.dir.unwrap_or(dir)),
                    prev_dir: dir,
                    point,
                    iteration: self.iteration + 1,
                });
            }
        }
        None
    }

    fn is_safe(&self, p: Point, ctx: &RobotContext) -> bool {
        p.x >= 0
            && p.y >= 0
            && p.x < ctx.size.width
            && p.y < ctx.size.height
            && self.is_safe_for_snake(&p, &ctx.snake)
            && ctx
                .opponents
                .iter()
                .all(|s| self.is_safe_for_opponent(&p, s))
    }

    fn is_safe_for_snake(&self, p: &Point, snake: &Snake) -> bool {
        if let Some(i) = Self::index_of_point_from_tail(snake, p) {
            i <= self.iteration
        } else {
            true
        }
    }

    fn is_safe_for_opponent(&self, p: &Point, snake: &Snake) -> bool {
        if !self.is_safe_for_snake(p, snake) {
            return false;
        }

        let head = snake.head();
        Direction::ALL
            .into_iter()
            .map(|dir| head.direction(dir))
            .all(|x| *p != x)
    }

    fn index_of_point_from_tail(snake: &Snake, p: &Point) -> Option<usize> {
        snake
            .points()
            .iter()
            .rev()
            .enumerate()
            .filter(|(_, x)| *x == p)
            .map(|(i, _)| i)
            .next()
    }
}

pub struct PathfindRobot {
    last: Mutex<Direction>,
}

impl PathfindRobot {
    pub fn new() -> Self {
        Self {
            last: Mutex::new(Direction::Up),
        }
    }

    fn find_path(
        &self,
        origin: Path,
        ctx: &RobotContext,
        max_iteration: usize,
        apple: bool,
    ) -> Path {
        let mut explored = HashSet::new();
        let mut paths = vec![origin.clone()];

        let mut highest_iteration: usize = 0;
        let mut highest_iteration_path: Path = origin;

        while let Some(mut path) = paths.pop() {
            if !explored.insert(path.point) {
                continue;
            }

            if apple && ctx.apples.contains(&path.point) {
                let final_path = self.find_path(path.fork(), ctx, 6, false);
                if final_path.iteration >= 6 {
                    return final_path;
                }
            }

            if path.iteration > highest_iteration {
                highest_iteration = path.iteration;
                highest_iteration_path = path.clone();
            }

            if path.iteration >= max_iteration {
                continue;
            }

            for new_path in path.advance_all(&ctx) {
                paths.insert(0, new_path);
            }
        }

        highest_iteration_path
    }
}

impl Robot for PathfindRobot {
    fn step(&self, ctx: RobotContext) -> Direction {
        let mut last = self.last.lock().unwrap();
        let path = self.find_path(Path::new(ctx.snake.head(), ctx.snake.direction()), &ctx, 40, true);
        *last = path.dir.unwrap_or(*last);
        *last
    }
}
