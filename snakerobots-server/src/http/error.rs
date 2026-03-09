use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub type RouteResult<T> = Result<T, RouteError>;

pub struct RouteError {
    code: StatusCode,
    error: String,
    message: String,
    report: Option<eyre::Report>,
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
}

impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        if let Some(report) = self.report {
            tracing::error!("{:?}", report);
        }

        #[derive(Debug, Serialize)]
        struct Payload {
            error: String,
            message: String,
        }

        (self.code, Json(Payload { error: self.error, message: self.message })).into_response()
    }
}

impl From<eyre::Report> for RouteError {
    fn from(value: eyre::Report) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal".into(),
            message: "Something went wrong".into(),
            report: Some(value),
        }
    }
}
