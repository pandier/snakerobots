use godot::meta::ToGodot;
use snakerobots_shared::dto;
use surf::StatusCode;

use crate::error::SrError;

#[derive(Debug)]
pub enum SrClientError {
    Surf(surf::Error),
    ResponseError(dto::Error),
    ResponseString(StatusCode, String),
    Unauthorized,
}

impl From<SrClientError> for SrError {
    fn from(value: SrClientError) -> Self {
        let (code, message) = match value {
            SrClientError::Surf(surf) => ("unknown".to_godot(), format!("{}", surf).to_godot()),
            SrClientError::ResponseString(code, err) => (
                "unknown".to_godot(),
                format!("{}: {}", code, err).to_godot(),
            ),
            SrClientError::ResponseError(err) => (err.error.to_godot(), err.message.to_godot()),
            SrClientError::Unauthorized => ("unauthorized".to_godot(), "Unauthorized".to_godot()),
        };
        Self { code, message }
    }
}

impl From<surf::Error> for SrClientError {
    fn from(value: surf::Error) -> Self {
        Self::Surf(value)
    }
}
