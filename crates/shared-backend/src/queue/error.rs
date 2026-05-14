

#[derive(Debug, thiserror::Error)]
pub enum MatchQueueError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
