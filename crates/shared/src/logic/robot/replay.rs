use crate::{Direction, logic::{Robot, RobotContext, robot::RobotResult}};

pub struct ReplayRobot {
    index: usize,
    moves: Vec<Direction>,
}

impl ReplayRobot {
    pub fn new(moves: Vec<Direction>) -> Self {
        Self {
            index: 0,
            moves,
        }
    }
}

impl Robot for ReplayRobot {
    fn step(&mut self, _ctx: RobotContext) -> RobotResult {
        if let Some(direction) = self.moves.get(self.index) {
            self.index += 1;
            Ok(*direction)
        } else {
            Err("replay finished".into())
        }
    }
}
