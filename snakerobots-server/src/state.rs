use eyre::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub dev_token: Option<String>,
    pub pg: PgPool,
}

impl AppState {
    pub async fn new() -> eyre::Result<AppState> {
        match dotenvy::dotenv() {
            Err(e) if !e.not_found() => Err(e).wrap_err("failed to load .env file")?,
            _ => {},
        }

        let dev_token = std::env::var("DEV_TOKEN").ok();

        let pg_url = std::env::var("DATABASE_URL")
            .wrap_err("couldn't interpret DATABASE_URL env")?;

        let pg = PgPoolOptions::new()
            .connect(&pg_url)
            .await
            .wrap_err("failed to connect to Postgres database")?;

        Ok(AppState { dev_token, pg })
    }
}
