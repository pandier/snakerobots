use std::collections::HashMap;

use eyre::Context;
use snakerobots_shared_backend::queue::{FinishedQueuedMatch, QueuedMatchDetails};

use crate::{service, state::AppState};

pub async fn queue_match(app: &AppState, details: QueuedMatchDetails) -> eyre::Result<()> {
    app.match_queue.queue(details).await?;
    Ok(())
}

pub async fn finalize_queued_matches(app: &AppState) -> eyre::Result<()> {
    let finished = app.match_queue.take_finished()
        .await
        .wrap_err("failed to take finished matches from queue")?;

    for m in finished {
        let id = m.id;

        finalize_queued_match(app, m)
            .await
            .wrap_err_with(|| format!("failed to finalize queued match for {}", id))?;
    }

    Ok(())
}

async fn finalize_queued_match(app: &AppState, m: FinishedQueuedMatch) -> eyre::Result<()> {
    // TODO: abort matches if conditions aren't met

    if let Some(replay) = m.result.replay {
        let mut elo = HashMap::new();

        if m.details.ranked {
            let snakes = &replay.snakes;
            if snakes.len() == 2 {
                let player1 = snakes[0].metadata;
                let player2 = snakes[1].metadata;

                let winner = replay.winner().map(|snake| snake.metadata.clone());
                let result = service::user::update_elo(&app, player1, player2, winner)
                    .await
                    .wrap_err("failed to update elo")?;

                elo.insert(player1, result.0);
                elo.insert(player2, result.1);
            }
        }

        service::matches::create_match(&app, m.id, m.queued_at, replay, elo)
            .await
            .wrap_err("failed to create match")?;
    }

    Ok(())
}
