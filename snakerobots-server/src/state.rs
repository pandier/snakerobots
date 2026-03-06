use eyre::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct AppState {
    pub pg: PgPool,
}

impl AppState {
    pub async fn new() -> eyre::Result<AppState> {
        let pg = PgPoolOptions::new()
            .connect("postgres://devuser:devpass@localhost:5432/snakerobots")
            .await
            .wrap_err("Failed to connect to Postgres database")?;

        Ok(AppState { pg })
    }
}
