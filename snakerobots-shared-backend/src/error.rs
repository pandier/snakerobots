
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // TODO: better error messages
    #[error("serde failed: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("redis failed: {0}")]
    Redis(#[from] redis::RedisError),
}
