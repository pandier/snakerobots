use std::str::FromStr;

use chrono::Duration;
use eyre::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub dev_token: Option<String>,
    pub session_timeout: Duration,
    pub pg: PgPool,
}

impl AppState {
    pub async fn new() -> eyre::Result<AppState> {
        match dotenvy::dotenv() {
            Err(e) if !e.not_found() => Err(e).wrap_err("failed to load .env file")?,
            _ => {},
        }

        let dev_token = env("DEV_TOKEN").ok();
        let session_timeout = Duration::seconds(env("SESSION_TIMEOUT")?);
        let pg_url = env::<String>("DATABASE_URL")?;

        let pg = PgPoolOptions::new()
            .connect(&pg_url)
            .await
            .wrap_err("failed to connect to Postgres database")?;

        Ok(AppState { dev_token, session_timeout, pg })
    }
}

fn env<T: FromStr>(key: &str) -> eyre::Result<T> where T::Err: std::fmt::Display {
    std::env::var(key)
        .map_err(|err| eyre::eyre!("{}", err))
        .and_then(|v| v.parse::<T>()
            .map_err(|err| eyre::eyre!("{}", err)))
        .wrap_err_with(|| format!("couldn't interpret {} env", key))
}
