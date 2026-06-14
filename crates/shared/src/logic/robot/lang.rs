use tropaion::interpreter::interpreter_builder::InterpreterBuilder;

use crate::{Direction, Point, logic::{Robot, robot::RobotResult}, lang::{
    error::{runtime_error::RuntimeError, context::ErrorContext},
    interpreter::interpreter::Interpreter,
    util::arg_convertor::ValueConvertable,
}};

use super::RobotContext;

pub const LIB_CODE: &str = r#"

struct Point(
    x: int,
    y: int,
);

struct Snake(
    points: Vec<Point>,
    direction: Direction,
) {
    pub fn head() -> Point {
        return this.points.get(0);
    }

    pub fn tail() -> Point {
        return this.points.get(this.points.size() - 1);
    }

    pub fn size() -> int {
        return this.points.size();
    }
}

struct Game(
    width: int,
    height: int,
    snake: Snake,
    opponents: Vec<Snake>,
    apples: Vec<Point>,
);
"#;

pub const DEFAULT_STACK_SIZE: usize = 1_000;
pub const DEFAULT_HEAP_SIZE: usize = 10_000_000;
pub const DEFAULT_MAX_INSTRUCTION_COST: usize = 10_000_000;

pub struct LangRobot {
    interpreter: Interpreter,
}

impl LangRobot {
    pub fn compile(mut code: String, stack_size: usize, heap_size: usize, max_inst_cost: usize) -> Result<Self, LangRobotError> {
        code += LIB_CODE;

        let compiled = crate::lang::compile(code)
            .map_err(|err| LangRobotError::Compile(err))?;
        let interpreter = InterpreterBuilder::new(compiled.clone())
            .stack_size(stack_size)
            .heap_size(heap_size)
            .max_instruction_cost(max_inst_cost)
            .build();

        Ok(Self {
            interpreter,
        })
    }
}

fn build_game_instance(ctx: &RobotContext) -> ValueConvertable {
    let mut s = crate::lang::util::arg_convertor::struct_convertor("Game");
    s.add_field(ctx.size.width);
    s.add_field(ctx.size.height);
    s.add_field(ctx.snake.clone());
    s.add_field(ctx.opponents.clone());
    s.add_field(ctx.apples.iter().cloned().collect::<Vec<Point>>());
    s.convert()
}

impl Robot for LangRobot {
    fn step(&mut self, ctx: RobotContext) -> RobotResult {
        let mut blob = crate::lang::run_compiled(
            &mut self.interpreter,
            "step",
            vec![
                build_game_instance(&ctx),
            ],
            &mut std::io::stdout(),
        )
        .map_err(|err| LangRobotError::Runtime(err.context))?;

        let dir_int = blob.next_int()
            .map_err(|e| LangRobotError::Return(e))?;

        blob.expect_end()
            .map_err(|e| LangRobotError::Return(e))?;

        let dir = match dir_int {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => return Err(Box::new(LangRobotError::InvalidDirection)),
        };

        Ok(dir)
    }
}

#[derive(Debug)]
pub enum LangRobotError {
    Compile(Vec<ErrorContext<Box<dyn std::error::Error>>>),
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
