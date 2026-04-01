use std::sync::Arc;

use eyre::Context;
use snakerobots_shared::{dto::GameReplay, logic::robot::lang::LangRobot};
use sqlx::types::Uuid;
use tokio::time::Instant;
use tracing::{debug, info, warn};

use crate::{service, state::AppState};

pub fn queue_game(app: Arc<AppState>, player1: Uuid, code1: String, player2: Uuid, code2: String) {
    tokio::spawn(async move {
        let _ = run_game(app, player1, code1, player2, code2).await;
    });
}

#[tracing::instrument(skip(app, code1, code2), err(Debug))]
async fn run_game(app: Arc<AppState>, player1: Uuid, code1: String, player2: Uuid, code2: String) -> eyre::Result<()> {
    let replay = tokio::task::spawn_blocking(move || {
        run_game_blocking(player1, code1, player2, code2)
    }).await?;

    if let Some(replay) = replay {
        service::matches::create_match(&app, replay).await
            .wrap_err("failed to create match")?;
    }

    Ok(())
}

#[tracing::instrument(skip(code1, code2))]
fn run_game_blocking(player1: Uuid, code1: String, player2: Uuid, code2: String) -> Option<GameReplay<Option<Uuid>>> {
    debug!("compiling code");

    let robot1 = LangRobot::compile(code1)
        .inspect_err(|err| warn!("failed to compile code of player 1: {}", err))
        .ok()?;
    let robot2 = LangRobot::compile(code2)
        .inspect_err(|err| warn!("failed to compile code of player 2: {}", err))
        .ok()?;

    debug!("running game");

    let start = Instant::now();

    let replay = GameReplay::run_standard(
        Box::new(robot1),
        Some(player1),
        Box::new(robot2),
        Some(player2)
    );

    info!(took = format!("{:?}", start.elapsed()), "finished running game");

    Some(replay)
}
