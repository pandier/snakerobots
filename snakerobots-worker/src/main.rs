mod runner;
mod state;

use std::{sync::Arc, time::Duration};

use eyre::WrapErr;
use snakerobots_shared_backend::processing::{PendingGame, PendingGameId};
use tokio::time::sleep;
use tracing::{error, info};

use crate::state::AppState;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = Arc::new(AppState::new().await?);
    
    loop {
        // TODO: only poll when resources are available

        let game = app.processing.poll_queue()
            .await
            .wrap_err("failed to poll queue")?;

        if let Some((id, game)) = game {
            tokio::spawn(process(app.clone(), id, game));
        }

        sleep(Duration::from_secs(15)).await;
    }
}

#[tracing::instrument(skip(app, game))]
async fn process(app: Arc<AppState>, id: PendingGameId, game: PendingGame) {
    info!("processing game");

    let finished_game = match runner::run_game(id.clone(), game).await {
        Ok(v) => v,
        Err(e) => {
            error!("failed to run game: {}", e);
            // TODO: feedback to redis processing
            return;
        }
    };

    match app.processing.finish(&finished_game).await {
        Ok(success) => {
            if success {
                info!("finished game");
            } else {
                info!("game was cancelled");
            }
        }
        Err(e) => {
            error!("failed to push finished game {}: {}", id, e);
        }
    }
}
