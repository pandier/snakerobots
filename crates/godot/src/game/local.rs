use godot::prelude::*;
use snakerobots_shared::logic::{self, Robot, robot::{error::PropagatingRobotErrorHandler, impls::PathfindRobot, lang::LangRobot}};
use crate::{SrResult, game::{error::SrLangError, timeline::GameTimeline}};

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
                .map_err(|err| super::error::convert_lang_error(&code, &err).to_variant())?;

            let opponent: Option<Box<dyn Robot>> = match opponent {
                SrLocalGameOpponent::Simple => Some(Box::new(PathfindRobot::new())),
                SrLocalGameOpponent::Mirror => Some(Box::new(LangRobot::compile(code.clone())
                    .map_err(|_| super::error::create_error("failed to compile mirror opponent").to_variant())?)),
                SrLocalGameOpponent::Code => Some(Box::new(LangRobot::compile(opponent_code)
                    .map_err(|_| super::error::create_error("failed to compile code opponent").to_variant())?)),
            };

            let game = logic::standard::create_standard_game_with_size(
                Box::new(robot),
                opponent,
                None,
                self.width,
                self.height,
            )
            .map_err(|_| SrLangError::create("invalid game size").to_variant())?;

            let timeline = GameTimeline::evaluate::<PropagatingRobotErrorHandler>(game)
                .map_err(|err| super::error::convert_error(&code, err).to_variant())?;

            Ok(Gd::from_object(timeline))
        })
    }
}
