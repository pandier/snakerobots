use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use snakerobots_shared::{dto::GameReplay, logic::robot::lang::LangRobot};
use sqlx::types::Uuid;
use tokio::{sync::mpsc, time::Instant};
use tracing::{debug, info, warn};

pub fn spawn_game(player1: Uuid, code1: String, player2: Uuid, code2: String, token: GameToken) {
    tokio::spawn(async move {
        let _ = run_game(player1, code1, player2, code2, token).await;
    });
}

#[tracing::instrument(skip(code1, code2, token))]
async fn run_game(player1: Uuid, code1: String, player2: Uuid, code2: String, mut token: GameToken) -> eyre::Result<()> {
    let cancelled = token.cancelled.clone();

    let replay = tokio::task::spawn_blocking(move || {
        run_game_blocking(player1, code1, player2, code2, &cancelled)
    }).await?;

    token.finish(replay);

    Ok(())
}

#[tracing::instrument(skip(code1, code2, cancelled))]
fn run_game_blocking(player1: Uuid, code1: String, player2: Uuid, code2: String, cancelled: &AtomicBool) -> Option<GameReplay<Uuid>> {
    debug!("compiling code");

    let robot1 = LangRobot::compile(code1)
        .inspect_err(|err| warn!("failed to compile code of player 1: {}", err))
        .ok()?;
    let robot2 = LangRobot::compile(code2)
        .inspect_err(|err| warn!("failed to compile code of player 2: {}", err))
        .ok()?;

    debug!("running game");

    let start = Instant::now();

    let replay = GameReplay::run_standard_cancellable(
        Box::new(robot1),
        player1,
        Box::new(robot2),
        player2,
        cancelled,
    );

    if replay.is_some() {
        info!(took = ?start.elapsed(), "finished running game");
    } else {
        info!(took = ?start.elapsed(), "cancelled game");
    }

    replay
}

pub fn game_token(id: Uuid, tx: mpsc::UnboundedSender<GameTokenFinish>) -> (GameToken, GameTokenHandle) {
    let cancelled = Arc::new(AtomicBool::new(false));
    let token = GameToken {
        id,
        tx,
        cancelled: cancelled.clone(),
        finished: false,
    };
    let handle = GameTokenHandle {
        id,
        cancelled,
    };
    (token, handle)
}

pub struct GameTokenFinish {
    pub id: Uuid,
    pub replay: Option<GameReplay<Uuid>>,
}

pub struct GameToken {
    pub id: Uuid,
    pub tx: mpsc::UnboundedSender<GameTokenFinish>,
    pub cancelled: Arc<AtomicBool>,
    pub finished: bool,
}

impl GameToken {
    pub fn finish(&mut self, replay: Option<GameReplay<Uuid>>) {
        if self.finished {
            return;
        }
        let _ = self.tx.send(GameTokenFinish {
            id: self.id,
            replay
        });
        self.finished = true;
    }
}

impl Drop for GameToken {
    fn drop(&mut self) {
        self.finish(None);
    }
}

pub struct GameTokenHandle {
    pub id: Uuid,
    pub cancelled: Arc<AtomicBool>,
}

impl GameTokenHandle {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
