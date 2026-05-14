use std::{env::VarError, str::FromStr, time::Duration};

use eyre::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

#[derive(Clone)]
pub struct AppConfig {
    pub worker_interval: Duration,
    pub max_game_count: usize,
    pub shutdown: CancellationToken,
    pub pg: PgPool,
}

impl AppConfig {
    pub async fn new() -> eyre::Result<AppConfig> {
        match dotenvy::dotenv() {
            Err(e) if !e.not_found() => Err(e).wrap_err("failed to load .env file")?,
            _ => {},
        }

        let worker_interval = Duration::from_secs(env::<u64>("WORKER_INTERVAL")?);
        let max_game_count = env::<usize>("WORKER_MAX_GAMES")?;

        let pg_url = env::<String>("DATABASE_URL")?;

        let pg = PgPoolOptions::new()
            .connect(&pg_url)
            .await
            .wrap_err("failed to connect to Postgres database")?;

        let shutdown = CancellationToken::new();
        tokio::spawn(shutdown_signal(shutdown.clone()));

        Ok(AppConfig {
            worker_interval,
            max_game_count,
            shutdown,
            pg,
        })
    }
}

async fn shutdown_signal(cancel: CancellationToken) {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => (),
            Err(e) => error!("failed to listen for ctrl_c signal: {}", e),
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => { s.recv().await; },
            Err(e) => error!("failed to listen for terminate signal: {}", e),
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutting down");
    cancel.cancel();
}

pub fn env<T: FromStr>(key: &str) -> eyre::Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    env_optional::<T>(key)
        .map(|v| v.ok_or_else(|| eyre::eyre!("missing environment variable {}", key)))
        .flatten()
}

pub fn env_optional<T: FromStr>(key: &str) -> eyre::Result<Option<T>>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(key) {
        Ok(s) => s.parse::<T>()
            .map(|v| Some(v))
            .map_err(|e| eyre::eyre!("failed to parse environment variable {}: {}", key, e)),
        Err(VarError::NotPresent) => Ok(None),
        Err(e) => Err(eyre::eyre!("failed to get environment variable {}: {}", key, e)),
    }
}
