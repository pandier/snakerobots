pub mod impls;
pub mod replay;

use std::collections::HashSet;

use crate::{Direction, Point, Size, logic::Snake};

pub trait Robot {
    fn step(&mut self, ctx: RobotContext) -> Direction;
}

pub struct RobotContext {
    pub size: Size,
    pub snake: Snake,
    pub opponents: Vec<Snake>,
    pub apples: HashSet<Point>,
}
