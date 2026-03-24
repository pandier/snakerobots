pub mod classes;
mod error;
mod middleware;

use arc_swap::ArcSwap;
use godot::prelude::*;
use snakerobots_shared::dto::{
    DefaultGameReplay, Match, MatchRequest, User, auth::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse}, match_request::{AcceptMatchRequest, CreateMatchRequest, DeleteMatchRequest}
};
use std::sync::Arc;
use surf::{
    Url,
    http::{convert::DeserializeOwned, headers::AUTHORIZATION},
};

use crate::{
    AsyncRuntime, SrFuture, SrResult,
    client::{classes::*, error::SrClientError, middleware::AuthorizationMiddleware},
};

type Token = Arc<ArcSwap<Option<String>>>;

#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct SrClient {
    client: surf::Client,
    token: Token,
    user: Option<User>,
    _base: Base<RefCounted>,
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
            _base,
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
            let res = this
                .client
                .post("/auth/login")
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
            let res = this
                .client
                .post("/auth/register")
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
            let res = this
                .client
                .get("/users/me")
                .header(AUTHORIZATION, format!("Bearer {}", &token))
                .parse_response_json::<User>()
                .await?;
            this.set_auth(Some(token), Some(res));
            Ok(())
        })
    }

    #[func]
    pub fn get_matches(&self, user_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            // TODO: injection o.o
            let res = self_gd.bind().client
                .get(format!("/users/{}/matches", user_id))
                .parse_response_json::<Vec<Match>>()
                .await?;
            Ok(res.into_iter()
                .map(|req| SrMatch::create(req))
                .collect::<Array<_>>())
        })
    }

    #[func]
    pub fn get_match(&self, match_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            // TODO: injection o.o
            let res = self_gd.bind().client
                .get(format!("/matches/{}", match_id))
                .parse_response_json::<Match>()
                .await?;
            Ok(SrMatch::create(res))
        })
    }

    #[func]
    pub fn get_match_replay(&self, match_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            // TODO: injection o.o
            let res = self_gd.bind().client
                .get(format!("/matches/{}/replay", match_id))
                .parse_response_json::<DefaultGameReplay>()
                .await?;
            Ok(SrMatchReplay::create(res))
        })
    }

    #[func]
    pub fn get_match_requests(&self) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let res = self_gd.bind().client
                .get("/match-requests")
                .parse_response_json::<Vec<MatchRequest>>()
                .await?;
            Ok(res.iter()
                .map(|req| SrMatchRequest::create(req))
                .collect::<Array<_>>())
        })
    }

    #[func]
    pub fn create_match_request(&self, username: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let req = CreateMatchRequest { username };
            let res = self_gd.bind().client
                .post("/match-requests")
                .body_json(&req)?
                .parse_response_json::<MatchRequest>()
                .await?;
            Ok(SrMatchRequest::create(&res))
        })
    }

    #[func]
    pub fn accept_match_request(&self, sender_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let req = AcceptMatchRequest { sender_id };
            let _ = self_gd.bind().client
                .post("/match-requests/accept")
                .body_json(&req)?
                .parse_response()
                .await?;
            Ok(())
        })
    }

    #[func]
    pub fn delete_match_request(&self, sender_id: String, receiver_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            self_gd.bind()._delete_match_request(sender_id, receiver_id).await
        })
    }

    pub async fn _delete_match_request(&self, sender_id: String, receiver_id: String) -> Result<(), SrClientError> {
        let req = DeleteMatchRequest { sender_id, receiver_id };
        let _ = self.client
            .delete("/match-requests")
            .body_json(&req)?
            .parse_response()
            .await?;
        // TODO: bool depending on status code (200 or 204)
        Ok(())
    }

    #[func]
    pub fn cancel_match_request(&self, receiver_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let this = self_gd.bind();
            let user = this.user.as_ref().ok_or(SrClientError::Unauthorized)?;
            this._delete_match_request(user.id.clone(), receiver_id).await
        })
    }

    #[func]
    pub fn decline_match_request(&self, sender_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let this = self_gd.bind();
            let user = this.user.as_ref().ok_or(SrClientError::Unauthorized)?;
            this._delete_match_request(sender_id, user.id.clone()).await
        })
    }

    fn set_auth(&mut self, token: Option<String>, user: Option<User>) {
        self.token.store(Arc::new(token));
        self.user = user;
    }

    #[func]
    pub fn get_token(&self) -> Variant {
        self.token
            .load()
            .as_ref()
            .as_ref()
            .map(|x| x.to_variant())
            .unwrap_or_else(Variant::nil)
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
    fn spawn_result<T>(
        &self,
        f: impl AsyncFnOnce(Gd<Self>) -> Result<T, SrClientError> + 'static,
    ) -> Gd<SrFuture>
    where
        T: ToGodot + 'static,
    {
        self.spawn_gd(async move |self_gd| SrResult::from(f(self_gd).await))
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
        let config = surf::Config::new().set_base_url(Url::parse(env!("API_URL"))?);

        #[cfg(feature = "dev-api")]
        let config = surf::Config::new()
            .set_base_url(Url::parse(env!("DEV_API_URL"))?)
            .add_header("X-Dev-Token", env!("DEV_TOKEN"))?;

        Ok(config.try_into()?)
    }
}

trait ParseResponse: Sized {
    async fn parse_response(self) -> Result<String, SrClientError>;
    async fn parse_response_json<T>(self) -> Result<T, SrClientError>
    where
        T: DeserializeOwned,
    {
        self.parse_response().await.and_then(|body| {
            serde_json::from_str::<T>(&body)
                .map_err(|err| SrClientError::from(surf::Error::from(err)))
        })
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
