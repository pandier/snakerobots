use std::error::Error;

use crate::{
    Direction,
    logic::{Robot, robot::RobotResult},
    lang::{
        compiler::compiler::CompilationResult,
        error::{runtime_error::RuntimeError, context::ErrorContext},
        util::arg_convertor::into_arg,
    },
};

use super::RobotContext;

#[derive(Debug, Clone)]
pub struct LangRobot {
    compiled: CompilationResult,
}

impl LangRobot {
    pub fn compile(code: String) -> Result<Self, LangRobotError> {
        let compiled = crate::lang::compile(code)
            .map_err(|err| LangRobotError::Compile(err))?;

        Ok(Self {
            compiled,
        })
    }
}

impl Robot for LangRobot {
    fn step(&mut self, ctx: RobotContext) -> RobotResult {
        let prev_dir = match ctx.snake.direction() {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        };

        let mut blob = crate::lang::run_compiled(
            self.compiled.clone(),
            "step",
            vec![
                into_arg(prev_dir),
            ],
            &mut std::io::empty(),
        )
        .map_err(|err| LangRobotError::Runtime(err))?;

        let dir_int = blob.next_int()
            .map_err(|e| LangRobotError::Return(e))?;

        blob.expect_end()
            .map_err(|e| LangRobotError::Return(e))?;

        let dir = match dir_int {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => return Err(Box::new(LangRobotError::InvalidDirection)),
        };

        Ok(dir)
    }
}

#[derive(Debug)]
pub enum LangRobotError {
    Compile(Vec<ErrorContext<Box<dyn Error>>>),
    Runtime(ErrorContext<RuntimeError>),
    Return(RuntimeError),
    InvalidDirection,
}

impl std::fmt::Display for LangRobotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compile(vec) => {
                if let Some(ctx) = vec.first() {
                    ctx.error.fmt(f)
                } else {
                    write!(f, "unknown error")
                }
            }
            Self::Runtime(ctx) => {
                ctx.error.fmt(f)
            }
            Self::Return(err) => {
                err.fmt(f)
            }
            Self::InvalidDirection => {
                write!(f, "invalid direction")
            }
        }
    }
}

impl std::error::Error for LangRobotError {}
