use std::sync::Arc;

use arc_swap::ArcSwap;
use godot::prelude::*;
use snakerobots_shared::dto::{self, User, auth::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse}};
use surf::{StatusCode, Url, http::{convert::DeserializeOwned, headers::AUTHORIZATION}, middleware::{Middleware, Next}};

use crate::{AsyncRuntime, SrFuture, SrResult, error::SrError};

type Token = Arc<ArcSwap<Option<String>>>;

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct SrClient {
    client: surf::Client,
    token: Token,
    user: Option<User>,
    _base: Base<RefCounted>
}

#[godot_api]
impl IRefCounted for SrClient {
    fn init(_base: Base<RefCounted>) -> Self {
        let token = Arc::new(ArcSwap::new(Arc::new(None)));
        let client = Self::create_client()
            .expect("failed to create http client")
            .with(AuthorizationMiddleware(token.clone()));
        Self {
            client,
            token,
            user: None,
            _base
        }
    }
}

#[godot_api]
impl SrClient {
    #[func]
    pub fn login(&self, username: String, password: String) -> Gd<SrFuture> {
        self.spawn_result(async move |mut self_gd| {
            let mut this = self_gd.bind_mut();
            let req = LoginRequest { username, password };
            let res = this.client.post("/auth/login")
                .body_json(&req)?
                .parse_response_json::<LoginResponse>()
                .await?;
            this.set_auth(Some(res.token), Some(res.user));
            Ok(())
        })
    }

    #[func]
    pub fn register(&self, username: String, password: String) -> Gd<SrFuture> {
        self.spawn_result(async move |mut self_gd| {
            let mut this = self_gd.bind_mut();
            let req = RegisterRequest { username, password };
            let res = this.client.post("/auth/register")
                .body_json(&req)?
                .parse_response_json::<RegisterResponse>()
                .await?;
            this.set_auth(Some(res.token), Some(res.user));
            Ok(())
        })
    }

    #[func]
    pub fn restore(&self, token: String) -> Gd<SrFuture> {
        self.spawn_result(async move |mut self_gd| {
            let mut this = self_gd.bind_mut();
            let res = this.client.get("/users/me")
                .header(AUTHORIZATION, format!("Bearer {}", &token))
                .parse_response_json::<User>()
                .await?;
            this.set_auth(Some(token), Some(res));
            Ok(())
        })
    }

    fn set_auth(&mut self, token: Option<String>, user: Option<User>) {
        self.token.store(Arc::new(token));
        self.user = user;
    }

    #[func]
    pub fn get_token(&self) -> Variant {
        self.token.load().as_ref().as_ref().map(|x| x.to_variant()).unwrap_or_else(Variant::nil)
    }

    #[func]
    pub fn get_me(&self) -> Option<Gd<SrUser>> {
        self.user.as_ref().map(SrUser::create)
    }

    #[func]
    pub fn is_logged_in(&self) -> bool {
        self.user.is_some()
    }

    #[inline]
    fn spawn_result<T>(&self, f: impl AsyncFnOnce(Gd<Self>) -> Result<T, SrClientError> + 'static) -> Gd<SrFuture>
    where
        T: ToGodot + 'static,
    {
        self.spawn_gd(async move |self_gd| {
            SrResult::from(f(self_gd).await)
        })
    }

    #[inline]
    fn spawn_gd<T>(&self, f: impl AsyncFnOnce(Gd<Self>) -> T + 'static) -> Gd<SrFuture>
    where
        T: ToGodot + 'static,
    {
        let self_gd = self.to_gd();
        AsyncRuntime::spawn_gd(async move { f(self_gd).await })
    }

    #[inline]
    fn create_client() -> Result<surf::Client, Box<dyn std::error::Error>> {
        #[cfg(not(feature = "dev-api"))]
        let config = surf::Config::new()
            .set_base_url(Url::parse(env!("API_URL"))?);

        #[cfg(feature = "dev-api")]
        let config = surf::Config::new()
            .set_base_url(Url::parse(env!("DEV_API_URL"))?)
            .add_header("X-Dev-Token", env!("DEV_TOKEN"))?;

        Ok(config.try_into()?)
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrUser {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
    #[var]
    pub created_at: i64,
}

#[godot_api]
impl SrUser {
    pub fn create(user: &User) -> Gd<Self> {
        Gd::from_object(Self {
            id: user.id.to_godot(),
            username: user.username.to_godot(),
            created_at: user.created_at.timestamp(),
        })
    }
}

struct AuthorizationMiddleware(Token);

#[surf::utils::async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn handle(&self, mut req: surf::Request, client: surf::Client, next: Next<'_>) -> surf::Result<surf::Response> {
        if req.header(AUTHORIZATION).is_none() {
            if let Some(token) = &**self.0.load() {
                req.append_header(AUTHORIZATION, format!("Bearer {}", token));
            }
        }
        next.run(req, client).await
    }
}

#[derive(Debug)]
enum SrClientError {
    Surf(surf::Error),
    ResponseError(dto::Error),
    ResponseString(StatusCode, String),
}

impl From<SrClientError> for SrError {
    fn from(value: SrClientError) -> Self {
        let (code, message) = match value {
            SrClientError::Surf(surf) => ("unknown".to_godot(), format!("{}", surf).to_godot()),
            SrClientError::ResponseString(code, err) => ("unknown".to_godot(), format!("{}: {}", code, err).to_godot()),
            SrClientError::ResponseError(err) => (err.error.to_godot(), err.message.to_godot()),
        };
        Self { code, message }
    }
}

impl From<surf::Error> for SrClientError {
    fn from(value: surf::Error) -> Self {
        Self::Surf(value)
    }
}

trait ParseResponse: Sized {
    async fn parse_response(self) -> Result<String, SrClientError>;
    async fn parse_response_json<T>(self) -> Result<T, SrClientError> where T: DeserializeOwned {
        self.parse_response().await
            .and_then(|body| serde_json::from_str::<T>(&body)
                .map_err(|err| SrClientError::from(surf::Error::from(err))))
    }
}

impl ParseResponse for surf::Response {
    async fn parse_response(mut self) -> Result<String, SrClientError> {
        let body = self.body_string().await?;
        if self.status().is_success() {
            Ok(body)
        } else if let Ok(error) = serde_json::from_str(&body) {
            Err(SrClientError::ResponseError(error))
        } else {
            Err(SrClientError::ResponseString(self.status(), body))
        }
    }
}

impl ParseResponse for surf::RequestBuilder {
    async fn parse_response(self) -> Result<String, SrClientError> {
        self.send().await?.parse_response().await
    }
}
