use axum::{Json, http::StatusCode, response::IntoResponse};
use snakerobots_shared::dto;
use tracing::error;
use validator::{ValidationErrors, ValidationErrorsKind};

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

    pub fn validation(errors: Box<ValidationErrors>) -> Self {
        let message = first_message_from_validation(errors).unwrap_or_else(|| String::from("Invalid input"));
        Self::new(StatusCode::BAD_REQUEST, "validation", &message)
    }
}

impl IntoResponse for RouteError {
    fn into_response(self) -> axum::response::Response {
        if let Some(report) = self.report {
            error!("{}", report);
        }

        (self.code, Json(dto::Error { error: self.error, message: self.message })).into_response()
    }
}

impl<T: Into<Box<dyn std::error::Error>>> From<T> for RouteError {
    fn from(value: T) -> Self {
        let e = value.into();
        match e.downcast::<ValidationErrors>() {
            Ok(e) => Self::validation(e),
            Err(e) => Self::internal(e),
        }
    }
}

fn first_message_from_validation(errors: Box<ValidationErrors>) -> Option<String> {
    for (_, k) in errors.into_errors() {
        match k {
            ValidationErrorsKind::Field(errors) => {
                for e in errors {
                    if let Some(message) = e.message {
                        return Some(message.into_owned());
                    }
                }
            },
            ValidationErrorsKind::List(list) => {
                for (_, errors) in list {
                    if let Some(message) = first_message_from_validation(errors) {
                        return Some(message);
                    }
                }
            },
            ValidationErrorsKind::Struct(errors) => {
                if let Some(message) = first_message_from_validation(errors) {
                    return Some(message);
                }
            },
        }
    }
    None
}
