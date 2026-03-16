use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("{0}")]
    Sqlx(sqlx::Error),
    #[error("{0}")]
    AlreadyExists(String),
    #[error("{0}")]
    LimitReached(String),
}

impl From<sqlx::Error> for ServiceError {
    fn from(value: sqlx::Error) -> Self {
        if let Some(database) = value.as_database_error() {
            if database.is_unique_violation() {
                return Self::AlreadyExists(database.message().to_owned());
            } else if database.code().as_deref() == Some("23Z01") {
                return Self::LimitReached(database.message().to_owned());
            }
        }
        Self::Sqlx(value)
    }
}
