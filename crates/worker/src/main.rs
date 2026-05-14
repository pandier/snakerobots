pub mod config;
pub mod worker;
pub mod runner;

use std::sync::Arc;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{config::AppConfig, worker::Worker};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let config = Arc::new(AppConfig::new().await?);

    let worker = Worker::new(config);

    worker.execution_loop().await;

    Ok(())
}
