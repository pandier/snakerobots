pub mod error;
pub mod impls;
#[cfg(feature = "lang")]
pub mod lang;
pub mod replay;

use std::collections::HashSet;

use crate::{Direction, Point, Size, logic::Snake};

pub type RobotResult = Result<Direction, Box<dyn std::error::Error>>;

pub trait Robot {
    fn step(&mut self, ctx: RobotContext) -> RobotResult;
}

pub struct RobotContext {
    pub size: Size,
    pub snake: Snake,
    pub opponents: Vec<Snake>,
    pub apples: HashSet<Point>,
}
