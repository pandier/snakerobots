use std::sync::Arc;

use eyre::Context;
use snakerobots_shared::dto;
use sqlx::types::Uuid;
use tracing::error;

use crate::{service, state::AppState};

pub fn queue_game(app: Arc<AppState>, player1: Uuid, player2: Uuid) {
    tokio::spawn(async move {
        let _ = run_game(app, player1, player2).await.inspect_err(|err| error!("{:?}", err));
    });
}

async fn run_game(app: Arc<AppState>, player1: Uuid, player2: Uuid) -> eyre::Result<()> {
    let replay = tokio::task::spawn_blocking(move || {
        dto::GameReplay::run_standard(Some(player1), Some(player2))
    }).await?;

    service::matches::create_match(&app, replay).await
        .wrap_err("failed to create match")?;

    Ok(())
}
