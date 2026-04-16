use godot::prelude::*;
use snakerobots_shared::{
    lang::{error::context::ErrorContext as LangErrorContext, util::either::Either},
    logic::{self, Robot, robot::{error::PropagatingRobotErrorHandler, impls::PathfindRobot, lang::{LangRobot, LangRobotError}}}
};
use crate::{SrResult, game::timeline::GameTimeline};

#[derive(GodotConvert, Var, Export, Default, Clone, Copy)]
#[godot(via = i32)]
pub enum SrLocalGameOpponent {
    #[default]
    Simple,
    Mirror,
    Code,
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrLocalGame {
    #[var]
    pub code: GString,
    #[var]
    pub opponent: SrLocalGameOpponent,
    #[var]
    pub opponent_code: GString,
    #[var]
    #[init(val = logic::standard::STANDARD_WIDTH)]
    pub width: i32,
    #[var]
    #[init(val = logic::standard::STANDARD_HEIGHT)]
    pub height: i32,
}

#[godot_api]
impl SrLocalGame {
    #[constant]
    const OPPONENT_SIMPLE: i32 = 0;
    #[constant]
    const OPPONENT_MIRROR: i32 = 1;
    #[constant]
    const OPPONENT_CODE: i32 = 2;

    #[func]
    pub fn run(&self) -> Gd<SrResult> {
        let code = String::from(&self.code);
        let opponent = self.opponent;
        let opponent_code = String::from(&self.opponent_code);

        SrResult::run(|| {
            let robot = LangRobot::compile(code.clone())
                .map_err(|err| convert_lang_error(&code, &err).to_variant())?;

            let opponent: Option<Box<dyn Robot>> = match opponent {
                SrLocalGameOpponent::Simple => Some(Box::new(PathfindRobot::new())),
                SrLocalGameOpponent::Mirror => Some(Box::new(LangRobot::compile(code.clone())
                    .map_err(|_| create_error("failed to compile mirror opponent").to_variant())?)),
                SrLocalGameOpponent::Code => Some(Box::new(LangRobot::compile(opponent_code)
                    .map_err(|_| create_error("failed to compile code opponent").to_variant())?)),
            };

            let game = logic::standard::create_standard_game_with_size(
                Box::new(robot),
                opponent,
                None,
                self.width,
                self.height,
            )
            .map_err(|_| SrLocalGameError::create("invalid game size").to_variant())?;

            let timeline = GameTimeline::evaluate::<PropagatingRobotErrorHandler>(game)
                .map_err(|err| convert_error(&code, err).to_variant())?;

            Ok(Gd::from_object(timeline))
        })
    }
}

fn convert_error(code: &str, err: Box<dyn std::error::Error>) -> Array<Gd<SrLocalGameError>> {
    if let Some(lang_err) = err.downcast_ref::<LangRobotError>() {
        return convert_lang_error(code, lang_err);
    }
    create_error(&err.to_string())
}

fn create_error(message: &str) -> Array<Gd<SrLocalGameError>> {
    vec![SrLocalGameError::create(message)].into_iter().collect()
}

fn convert_lang_error(code: &str, err: &LangRobotError) -> Array<Gd<SrLocalGameError>> {
    match err {
        LangRobotError::Compile(errs) => {
            errs.into_iter()
                .map(|ctx| convert_error_context(code, ctx))
                .collect()
        },
        LangRobotError::Runtime(ctx) => {
            vec![convert_error_context(code, ctx)].into_iter().collect()
        },
        err => {
            vec![SrLocalGameError::create(&err.to_string())].into_iter().collect()
        }
    }
}

fn convert_error_context<T>(code: &str, ctx: &LangErrorContext<T>) -> Gd<SrLocalGameError>
where 
    T: std::fmt::Display,
{
    Gd::from_object(SrLocalGameError {
        span: SrLocalGameErrorSpan::from_context(ctx, code).map(|x| Gd::from_object(x)),
        message: ctx.error.to_string().to_godot(),
    })
}

#[derive(GodotClass, Default)]
#[class(init, base=RefCounted)]
pub struct SrLocalGameError {
    #[var]
    pub span: Option<Gd<SrLocalGameErrorSpan>>,
    #[var]
    pub message: GString,
}

impl SrLocalGameError {
    pub fn create(message: &str) -> Gd<SrLocalGameError> {
        Gd::from_object(Self {
            span: None,
            message: message.to_godot(),
        })
    }
}

#[derive(GodotClass, Default)]
#[class(init, base=RefCounted)]
pub struct SrLocalGameErrorSpan {
    #[var]
    pub line_from: i64,
    #[var]
    pub line_to: i64,
    #[var]
    pub column_from: i64,
    #[var]
    pub column_to: i64,
    #[var]
    pub char_from: i64,
    #[var]
    pub char_to: i64,
}

impl SrLocalGameErrorSpan {
    pub fn from_context<T>(ctx: &LangErrorContext<T>, code: &str) -> Option<Self> {
        match ctx.span {
            Either::Left(span) => {
                let (line_from, column_from) = get_line_and_column(code, span.from)?;
                let (line_to, column_to) = get_line_and_column(code, span.to)?;

                Some(Self {
                    line_from,
                    line_to,
                    column_from,
                    column_to,
                    char_from: span.from as i64,
                    char_to: span.to as i64,
                })
            },
            Either::Right(l) => {
                let mut i = 0;
                let mut c = 0;
                let mut len = 0;

                for line in code.split_inclusive('\n') {
                    c += len;
                    len = line.chars().count();

                    if i == l {
                        break
                    }

                    i += 1;
                }

                // check if outside the bounds of the code
                if i < l {
                    None
                } else {
                    Some(Self {
                        line_from: l as i64,
                        line_to: l as i64,
                        column_from: 0,
                        column_to: len as i64,
                        char_from: c as i64,
                        char_to: (c + len) as i64,
                    })
                }
            },
        }
    }
}

fn get_line_and_column(s: &str, i: usize) -> Option<(i64, i64)> {
    let mut j = 0usize;
    let mut line = 0;
    let mut col = 0;

    for char in s.chars() {
        if j == i {
            break;
        }

        j += 1;
        col += 1;
        if char == '\n' {
            line += 1;
            col = 0;
        }
    }

    // check if outside the bounds of the code
    if j < i {
        None
    } else {
        Some((line, col))
    }
}
