use eyre::WrapErr;
use snakerobots_shared_backend::processing::RedisGameProcessing;
use tracing::info;

pub struct AppState {
    pub processing: RedisGameProcessing,
}

impl AppState {
    pub async fn new() -> eyre::Result<Self> {
        let redis = redis::Client::open("redis://127.0.0.1/")
            .wrap_err("failed to connect to Redis")?;

        let redis_con = redis.get_multiplexed_async_connection()
            .await
            .wrap_err("failed to get async Redis connection")?;

        info!("connected to Redis");

        let processing = RedisGameProcessing::new(redis_con);

        Ok(Self {
            processing,
        })
    }
}
