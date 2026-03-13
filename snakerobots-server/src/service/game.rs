use std::sync::Arc;

use eyre::{Context, ContextCompat};
use rand::RngExt;
use snakerobots_shared::{Direction, Point, Size, dto, logic::{self, robot::impls::PathfindRobot}};
use sqlx::types::Uuid;

use crate::{service, state::AppState};

pub fn queue_game(app: Arc<AppState>, player1: Uuid, player2: Uuid) {
    tokio::spawn(async move {
        let _ = run_game(app, player1, player2).await.inspect_err(|err| tracing::error!("{:?}", err));
    });
}

async fn run_game(app: Arc<AppState>, player1: Uuid, player2: Uuid) -> eyre::Result<()> {
    let game = tokio::task::spawn_blocking(run_game_blocking).await?
        .wrap_err("failed to run game")?;

    let mut snakes = Vec::new();
    snakes.push((game.snakes.get(0).ok_or_else(|| eyre::eyre!("missing snake for player 1"))?, player1));
    snakes.push((game.snakes.get(1).ok_or_else(|| eyre::eyre!("missing snake for player 2"))?, player2));

    service::matches::create_match(&app, game.seed, game.result.winner(), snakes).await
        .wrap_err("failed to create match")?;

    Ok(())
}

fn run_game_blocking() -> eyre::Result<dto::Game> {
    let seed = rand::rng().random();

    let width = 25;
    let height = 13;

    let players: Vec<logic::Player> = vec![
        (Point::new(1, height / 2), Direction::Right),
        (Point::new(width - 2, height / 2), Direction::Left),
    ]
    .into_iter()
    .map(|(p, d)| {
        let mut snake = logic::Snake::new(p, d);
        snake.expand_head(d);
        logic::Player::new(snake, Box::new(PathfindRobot::new()))
    })
    .collect();

    let mut snakes: Vec<dto::GameSnake> = players
        .iter()
        .map(|_| dto::GameSnake { moves: Vec::new() })
        .collect();

    let mut game = logic::Game::new(Size::new(width, height), 1, seed, players)
        .wrap_err("predefined layout should be correct")?;

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
        seed,
        snakes,
        result,
    })
}
