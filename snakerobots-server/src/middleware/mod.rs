pub mod dev_token;

use axum::{extract::{Request, State}, http::StatusCode, middleware::Next, response::Response};

pub async fn dev_token(
    State(dev_token): State<String>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match req.headers().get("x-dev-token") {
        Some(value) if *value == dev_token => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::FORBIDDEN),
    }
}
