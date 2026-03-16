use axum::{Json, http::StatusCode, response::IntoResponse};
use snakerobots_shared::dto;

pub type RouteResult<T> = Result<T, RouteError>;

pub struct RouteError {
    code: StatusCode,
    error: String,
    message: String,
    report: Option<Box<dyn std::error::Error>>,
}

impl RouteError {
    pub fn new(code: StatusCode, error: &str, message: &str) -> Self {
        Self {
            code,
            error: error.into(),
            message: message.into(),
            report: None,
        }
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", "Not Found")
    }

    pub fn internal(report: Box<dyn std::error::Error>) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal".into(),
            message: "Something went wrong".into(),
            report: Some(report),
        }
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        if let Some(report) = self.report {
            tracing::error!("{}", report);
        }

        (self.code, Json(dto::Error { error: self.error, message: self.message })).into_response()
    }
}


impl<T: Into<Box<dyn std::error::Error>>> From<T> for RouteError {
    fn from(value: T) -> Self {
        Self::internal(value.into())
    }
}
