use axum::{body::Body, http::{Request, Response, StatusCode}, response::IntoResponse};
use tower_http::validate_request::ValidateRequest;

#[derive(Clone)]
pub struct DevTokenHeader(pub String);

impl<B> ValidateRequest<B> for DevTokenHeader {
    type ResponseBody = Body;

    fn validate(
        &mut self,
        request: &mut Request<B>,
    ) -> Result<(), Response<Self::ResponseBody>> {
        match request.headers().get("x-dev-token") {
            Some(value) if *value == self.0 => Ok(()),
            _ => Err(StatusCode::FORBIDDEN.into_response()),
        }
    }
}
