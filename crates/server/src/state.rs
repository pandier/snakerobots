use std::{env::VarError, str::FromStr};

use chrono::Duration;
use eyre::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

#[derive(Clone)]
pub struct AppState {
    pub dev_token: Option<String>,
    pub session_timeout: Duration,
    pub shutdown: CancellationToken,
    pub pg: PgPool,
}

impl AppState {
    pub async fn new() -> eyre::Result<AppState> {
        match dotenvy::dotenv() {
            Err(e) if !e.not_found() => Err(e).wrap_err("failed to load .env file")?,
            _ => {},
        }

        let dev_token = env_optional("DEV_TOKEN")?;
        let session_timeout = Duration::seconds(env("SESSION_TIMEOUT")?);
        let pg_url = env::<String>("DATABASE_URL")?;

        let pg = PgPoolOptions::new()
            .connect(&pg_url)
            .await
            .wrap_err("failed to connect to Postgres database")?;

        let shutdown = CancellationToken::new();
        tokio::spawn(shutdown_signal(shutdown.clone()));

        Ok(AppState {
            dev_token,
            session_timeout,
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
