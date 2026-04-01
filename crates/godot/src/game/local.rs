use godot::prelude::*;
use snakerobots_shared::{lang::{error::context::ErrorContext as LangErrorContext, util::either::Either}, logic::{self, robot::{error::PropagatingRobotErrorHandler, impls::PathfindRobot, lang::{LangRobot, LangRobotError}}}};

use crate::{SrResult, game::timeline::GameTimeline};

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrLocalGame {
    #[var]
    pub code: GString,
    #[var]
    #[init(val = logic::standard::STANDARD_WIDTH)]
    pub width: i32,
    #[var]
    #[init(val = logic::standard::STANDARD_HEIGHT)]
    pub height: i32,
}

#[godot_api]
impl SrLocalGame {

    #[func]
    pub fn run(&self) -> Gd<SrResult> {
        let code = String::from(&self.code);

        SrResult::run(|| {
            let robot = LangRobot::compile(code.clone())
                .map_err(|err| convert_lang_error(&code, &err).to_variant())?;

            let game = logic::standard::create_standard_game_with_size(
                Box::new(robot),
                Box::new(PathfindRobot::new()),
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
    vec![SrLocalGameError::create(&err.to_string())].into_iter().collect()
}

fn convert_lang_error(code: &str, err: &LangRobotError) -> Array<Gd<SrLocalGameError>> {
    match err {
        LangRobotError::Compile(errs) => {
            return errs.into_iter()
                .map(|ctx| convert_error_context(code, ctx))
                .collect()
        },
        LangRobotError::Runtime(ctx) => {
            return vec![convert_error_context(code, ctx)].into_iter().collect();
        },
        err => {
            return vec![SrLocalGameError::create(&err.to_string())].into_iter().collect();
        }
    }
}

fn convert_error_context<T>(code: &str, ctx: &LangErrorContext<T>) -> Gd<SrLocalGameError>
where 
    T: std::fmt::Display,
{
    Gd::from_object(SrLocalGameError {
        span: Some(Gd::from_object(SrLocalGameErrorSpan::from_context(ctx, code))),
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
    pub fn from_context<T>(ctx: &LangErrorContext<T>, code: &str) -> Self {
        return match ctx.span {
            Either::Left(span) => {
                let (line_from, column_from) = get_line_and_column(code, span.from);
                let (line_to, column_to) = get_line_and_column(code, span.to);

                Self {
                    line_from,
                    line_to,
                    column_from,
                    column_to,
                    char_from: span.from as i64,
                    char_to: span.to as i64,
                }
            },
            Either::Right(l) => {
                let mut c = 0;
                let mut len = 0;

                for (i, line) in code.split_inclusive('\n').enumerate() {
                    c += len;
                    len = line.chars().count();

                    if i == l {
                        break
                    }
                }

                Self {
                    line_from: l as i64,
                    line_to: l as i64,
                    column_from: 0,
                    column_to: len as i64,
                    char_from: c as i64,
                    char_to: (c + len) as i64,
                }
            },
        }
    }

}

fn get_line_and_column(s: &str, i: usize) -> (i64, i64) {
    let mut line = 0;
    let mut col = 0;

    for (j, char) in s.chars().enumerate() {
        if j == i {
            break;
        }

        col += 1;
        if char == '\n' {
            line += 1;
            col = 0;
        }
    }

    (line, col)
}
