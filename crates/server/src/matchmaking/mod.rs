use std::sync::Arc;

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

        info!("matching {} entries", entries.len());

        let mut queue = MatchmakingQueue::new(entries);

        while let Some((a_entry, b_entry)) = queue.next_match() {
            let _ = self.create_match(a_entry, b_entry)
                .await
                .inspect_err(|e| error!("failed to match {} with {}: {:#}", a_entry.id, b_entry.id, e));
        }

        Ok(())
    }

    #[instrument(skip_all, fields(a = %a_entry.id, b = %b_entry.id))]
    async fn create_match(&self, a_entry: &MatchmakingEntry, b_entry: &MatchmakingEntry) -> eyre::Result<()> {
        debug!("matched");

        let Some(a_code) = service::robot::download_robot(&self.app, a_entry.id, a_entry.robot_id).await? else {
            debug!("skipping because robot for A is missing");
            return Ok(());
        };

        let Some(b_code) = service::robot::download_robot(&self.app, b_entry.id, b_entry.robot_id).await? else {
            debug!("skipping because robot for B is missing");
            return Ok(());
        };

        service::game::queue_game(self.app.clone(), a_entry.id, a_code, b_entry.id, b_code, true);

        Ok(())
    }
}
