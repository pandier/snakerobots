use std::sync::Arc;

use eyre::Context;
use snakerobots_shared_backend::queue::{QueuedMatchDetails, QueuedMatchDetailsPlayer};
use tokio::time::{Instant, sleep};
use tracing::{debug, error, info, instrument};

use crate::{matchmaking::queue::{MatchmakingEntry, MatchmakingQueue}, service, state::AppState};

pub mod queue;

pub struct Matchmaker {
    app: Arc<AppState>,
}

impl Matchmaker {

    pub fn new(app: Arc<AppState>) -> Self {
        Self { app }
    }

    pub async fn start(self) {
        if let Some(interval) = self.app.matchmaker_interval.clone() {
            info!("running matchmaker every {:?}", interval);

            let mut start = Instant::now();

            loop {
                tokio::select! {
                    _ = sleep(interval.saturating_sub(Instant::now().duration_since(start))) => {},
                    _ = self.app.shutdown.cancelled() => break,
                }

                start = Instant::now();

                let _ = self.execute().await
                    .inspect_err(|e| error!("failed to execute matchmaker: {:#}", e));
            }
        }
    }

    pub async fn execute(&self) -> eyre::Result<()> {
        let entries = service::user::get_users_for_matchmaking(&self.app, 0, 100)
            .await?
            .into_iter()
            .map(|(id, elo, robot_id)| MatchmakingEntry::new(id, elo, robot_id))
            .collect::<Vec<_>>();

        debug!("executing matchmaker for {} entries", entries.len());

        let mut queue = MatchmakingQueue::new(entries);
        let mut count = 0usize;

        while let Some((a_entry, b_entry)) = queue.next_match() {
            count += 1;

            let _ = self.create_match(a_entry, b_entry)
                .await
                .inspect_err(|e| error!("failed to match {} with {}: {:#}", a_entry.id, b_entry.id, e));
        }

        if count > 0 {
            info!("matched {} entries", count * 2);
        }

        Ok(())
    }

    #[instrument(skip_all, fields(a = %a_entry.id, b = %b_entry.id))]
    async fn create_match(&self, a_entry: &MatchmakingEntry, b_entry: &MatchmakingEntry) -> eyre::Result<()> {
        debug!("matched");

        let details = QueuedMatchDetails {
            ranked: true,
            players: vec![
                QueuedMatchDetailsPlayer {
                    id: a_entry.id,
                    robot_id: a_entry.robot_id,
                },
                QueuedMatchDetailsPlayer {
                    id: b_entry.id,
                    robot_id: b_entry.robot_id,
                },
            ],
        };

        service::match_queue::queue_match(&self.app, details)
            .await
            .wrap_err("failed to queue match")?;

        Ok(())
    }
}
