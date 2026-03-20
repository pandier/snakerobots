use crate::{Direction, logic::{Robot, RobotContext}};

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
    fn step(&mut self, _ctx: RobotContext) -> Direction {
        let direction = self.moves.get(self.index)
            .cloned()
            .unwrap_or_else(|| self.moves[0]);
        self.index += 1;
        direction
    }
}
