use std::sync::Arc;

use eyre::Context;
use snakerobots_shared::{dto, logic::{self}};
use sqlx::types::Uuid;
use tracing::error;

use crate::{service, state::AppState};

pub fn queue_game(app: Arc<AppState>, player1: Uuid, player2: Uuid) {
    tokio::spawn(async move {
        let _ = run_game(app, player1, player2).await.inspect_err(|err| error!("{:?}", err));
    });
}

async fn run_game(app: Arc<AppState>, player1: Uuid, player2: Uuid) -> eyre::Result<()> {
    let game = tokio::task::spawn_blocking(run_game_blocking).await?
        .wrap_err("failed to run game")?;

    let winner = match game.result.winner() {
        Some(0) => Some(player1),
        Some(1) => Some(player2),
        _ => None,
    };

    service::matches::create_match(&app, game.seed, winner, vec![player1, player2]).await
        .wrap_err("failed to create match")?;

    Ok(())
}

fn run_game_blocking() -> eyre::Result<dto::Game> {
    let mut game = logic::standard::create_standard_game();

    let mut snakes: Vec<dto::GameSnake> = game.players().iter()
        .map(|_| dto::GameSnake { moves: Vec::new() })
        .collect();

    let result = loop {
        match game.step() {
            logic::GameStep::Success { moves, added_apples: _, removed_apples: _ } => {
                for (i, dir) in moves {
                    if let Some(snake) = snakes.get_mut(i) {
                        snake.moves.push(dir);
                    }
                }
            }
            logic::GameStep::Finished(result) => break result,
        }
    };

    Ok(dto::Game {
        seed: game.seed(),
        snakes,
        result,
    })
}
