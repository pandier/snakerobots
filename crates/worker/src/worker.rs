use std::{collections::HashMap, sync::Arc, time::Duration};

use eyre::Context;
use snakerobots_shared_backend::queue::{MatchQueue, QueuedMatchDetails, QueuedMatchDetailsPlayer, QueuedMatchResult};
use tokio::{sync::mpsc, time::sleep};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{config::AppConfig, runner::{GameToken, GameTokenFinish, GameTokenHandle}};

pub struct Worker {
    config: Arc<AppConfig>,
    queue: Arc<MatchQueue>,
    worker_id: Uuid,
    games: Vec<GameTokenHandle>,
    finish_rx: mpsc::UnboundedReceiver<GameTokenFinish>,
    finish_tx: mpsc::UnboundedSender<GameTokenFinish>,
}

impl Worker {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let (finish_tx, finish_rx) = mpsc::unbounded_channel();
        Self {
            queue: Arc::new(MatchQueue::new(config.pg.clone())),
            config,
            worker_id: Uuid::new_v4(),
            games: Vec::new(),
            finish_rx,
            finish_tx,
        }
    }

    pub async fn execution_loop(mut self) {
        info!("starting execution loop with worker id {}", self.worker_id);

        loop {
            self.execute()
                .await
                .inspect_err(|e| error!("failed to execute worker: {e:#}"))
                .ok();

            tokio::select! {
                _ = sleep(self.config.worker_interval) => {},
                _ = self.config.shutdown.cancelled() => break,
                finish = self.finish_rx.recv() => {
                    if let Some(finish) = finish {
                        let id = finish.id;
                        self.handle_game_finish(finish)
                            .await
                            .inspect_err(|e| error!("failed to handle game finish of {id}: {e:#}"))
                            .ok();
                    } else {
                        info!("stopping execution loop because of closed finish channel");
                        break;
                    }
                }
            }
        }

        // cancel running games
        for handle in self.games {
            handle.cancel();
        }
    }

    async fn execute(&mut self) -> eyre::Result<()> {
        if !self.games.is_empty() {
            debug!("sending heartbeat for games");

            for handle in &self.games {
                let id = handle.id;
                self.queue.update(id, self.worker_id, self.expire_duration())
                    .await
                    .inspect_err(|e| error!("failed to send heartbeat for {id}: {e:#}"))
                    .ok();
            }
        }

        debug!("checking queue for new games");

        let mut count = 0usize;

        while self.games.len() < self.config.max_game_count {
            let queued_match = self.queue.take(self.worker_id, self.expire_duration())
                .await
                .wrap_err("failed to take from match queue")?;

            if let Some(queued_match) = queued_match {
                match self.start_game(queued_match.id, queued_match.details).await {
                    Ok(success) => {
                        if !success {
                            self.queue.remove(queued_match.id, self.worker_id)
                                .await
                                .inspect_err(|e| error!("failed to remove skipped game from queue: {e:#}"))
                                .ok();
                        } else {
                            count += 1;
                        }
                    }
                    Err(e) => error!("failed to start game: {e:#}")
                }
            } else {
                break;
            }
        }

        if count > 0 {
            info!("started execution of {count} games");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn start_game(&mut self, id: Uuid, details: QueuedMatchDetails) -> eyre::Result<bool> {
        let players = self.fetch_code(details.players)
            .await
            .wrap_err("failed to fetch code of players")?;

        let Ok(slice) = <Vec<_> as TryInto<[(Uuid, Option<String>); 2]>>::try_into(players) else {
            warn!("skipping game because of incorrect player count");
            return Ok(false);
        };

        let [(player1, code1), (player2, code2)] = slice;

        let Some(code1) = code1 else {
            warn!("skipping game because of missing code for player 1");
            return Ok(false);
        };

        let Some(code2) = code2 else {
            warn!("skipping game because of missing code for player 2");
            return Ok(false);
        };

        info!("running game");

        crate::runner::spawn_game(player1, code1, player2, code2, self.create_game_token(id));

        Ok(true)
    }

    async fn fetch_code(&self, players: Vec<QueuedMatchDetailsPlayer>) -> eyre::Result<Vec<(Uuid, Option<String>)>> {
        let robots: Vec<(Uuid, String)> = sqlx::query_as(r#"
            SELECT id, code
            FROM robots r
            WHERE id = ANY($1)
        "#)
        .bind(players.iter().map(|x| x.robot_id).collect::<Vec<_>>())
        .fetch_all(&self.config.pg)
        .await?;

        let mut robots = robots.into_iter().collect::<HashMap<_, _>>();

        let players = players.into_iter()
            .map(|player| (player.id, robots.remove(&player.robot_id)))
            .collect();

        Ok(players)
    }
    
    async fn handle_game_finish(&mut self, finish: GameTokenFinish) -> eyre::Result<()> {
        debug!("received finish for id {}", finish.id);

        self.games.retain(|x| x.id != finish.id);

        self.queue.finish(finish.id, self.worker_id, QueuedMatchResult { replay: finish.replay })
            .await
            .wrap_err("failed to finish game in queue")?;

        Ok(())
    }

    fn create_game_token(&mut self, id: Uuid) -> GameToken {
        let (token, handle) = crate::runner::game_token(id, self.finish_tx.clone());
        self.games.push(handle);
        token
    }

    fn expire_duration(&self) -> Duration {
        self.config.worker_interval + Duration::from_secs(30)
    }
}
