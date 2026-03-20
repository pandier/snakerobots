use surf::{
    Client, Request, Response,
    http::headers::AUTHORIZATION,
    middleware::{Middleware, Next},
};

use crate::client::Token;

pub struct AuthorizationMiddleware(pub Token);

#[surf::utils::async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        client: Client,
        next: Next<'_>,
    ) -> surf::Result<Response> {
        if req.header(AUTHORIZATION).is_none() {
            if let Some(token) = &**self.0.load() {
                req.append_header(AUTHORIZATION, format!("Bearer {}", token));
            }
        }
        next.run(req, client).await
    }
}
