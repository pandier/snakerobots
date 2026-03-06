#[cfg(feature = "robot-impls")]
pub mod impls;

use std::collections::HashSet;

use crate::{Direction, Point, Size, logic::Snake};

pub trait Robot {
    fn step(&self, ctx: RobotContext) -> Direction;
}

pub struct RobotContext {
    pub size: Size,
    pub snake: Snake,
    pub opponents: Vec<Snake>,
    pub apples: HashSet<Point>,
}
