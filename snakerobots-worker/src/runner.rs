use eyre::ensure;
use snakerobots_shared::logic::{self, GameStep};
use snakerobots_shared_backend::processing::{FinishedGame, FinishedGamePlayer, PendingGame, PendingGameId};

pub async fn run_game(id: PendingGameId, pending: PendingGame) -> eyre::Result<FinishedGame> {
    Ok(tokio::task::spawn_blocking(move || {
        run_game_blocking(id, pending)
    }).await??)
}

fn run_game_blocking(id: PendingGameId, pending: PendingGame) -> eyre::Result<FinishedGame> {
    let mut game = logic::standard::create_standard_game();

    ensure!(game.snakes().len() == pending.players.len(), "provided player size does not match the standard game");

    let mut players: Vec<FinishedGamePlayer> = pending.players.into_iter()
        .map(|id| FinishedGamePlayer { id, moves: Vec::new() })
        .collect();

    let result = loop {
        match game.step() {
            GameStep::Success { moves, added_apples: _, removed_apples: _ } => {
                for (i, dir) in moves {
                    if let Some(snake) = players.get_mut(i) {
                        snake.moves.push(dir);
                    }
                }
            }
            GameStep::Finished(result) => break result,
        }
    };

    Ok(FinishedGame {
        id,
        seed: game.seed(),
        players,
        result,
    })
}