pub mod classes;
mod error;
mod middleware;

use arc_swap::ArcSwap;
use godot::prelude::*;
use snakerobots_shared::dto::{self, DefaultGameReplay, Match, MatchRequest, PrivateUser, auth::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse}, match_request::{AcceptMatchRequest, CreateMatchRequest, DeleteMatchRequest}, UpdateCompetingRobot};
use std::sync::Arc;
use surf::{
    Url,
    http::{convert::DeserializeOwned, headers::AUTHORIZATION},
};
use snakerobots_shared::dto::robot::CreateRobot;
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
    user: Option<PrivateUser>,
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
                .parse_response_json::<PrivateUser>()
                .await?;
            this.set_auth(Some(token), Some(res));
            Ok(())
        })
    }

    #[func]
    pub fn logout(&mut self) -> Gd<SrFuture> {
        self.spawn_result(async move |mut self_gd| {
            let mut this = self_gd.bind_mut();
            let res = this
                .client
                .post("/auth/logout")
                .parse_response()
                .await;
            this.set_auth(None, None);
            res?; // try now so that auth is always set to None
            Ok(())
        })
    }

    fn set_auth(&mut self, token: Option<String>, user: Option<PrivateUser>) {
        self.token.store(Arc::new(token));
        self.user = user;
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
    pub fn create_match_request(&self, username: String, robot_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let req = CreateMatchRequest { username, robot_id };
            let res = self_gd.bind().client
                .post("/match-requests")
                .body_json(&req)?
                .parse_response_json::<MatchRequest>()
                .await?;
            Ok(SrMatchRequest::create(&res))
        })
    }

    #[func]
    pub fn accept_match_request(&self, sender_id: String, robot_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let req = AcceptMatchRequest { sender_id, robot_id };
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
            let user = this.user.as_ref().ok_or(SrClientError::unauthorized())?;
            this._delete_match_request(user.id.clone(), receiver_id).await
        })
    }

    #[func]
    pub fn decline_match_request(&self, sender_id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let this = self_gd.bind();
            let user = this.user.as_ref().ok_or(SrClientError::unauthorized())?;
            this._delete_match_request(sender_id, user.id.clone()).await
        })
    }

    #[func]
    pub fn get_user(&self, id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let res = self_gd.bind().client
                .get(format!("/users/{}", id))
                .parse_response_json::<dto::User>()
                .await?;
            Ok(SrUser::create(&res))
        })
    }

    #[func]
    pub fn get_user_ranking(&self, id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let res = self_gd.bind().client
                .get(format!("/users/{}/ranking", id))
                .parse_response_json::<dto::UserRanking>()
                .await?;
            Ok(res.rank)
        })
    }

    #[func]
    pub fn get_robots(&self) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let res = self_gd.bind().client
                .get("/robots")
                .parse_response_json::<Vec<dto::Robot>>()
                .await?;
            Ok(res.iter()
                .map(|robot| SrRobot::create(robot))
                .collect::<Array<_>>())
        })
    }

    #[func]
    pub fn get_robot(&self, id: String) -> Gd<SrFuture> {
        // TODO: injection o.o
        self.spawn_result(async move |self_gd| {
            let res = self_gd.bind().client
                .get(format!("/robots/{}", id))
                .parse_response_json::<dto::Robot>()
                .await?;
            Ok(SrRobot::create(&res))
        })
    }

    #[func]
    pub fn create_robot(&self, name: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let req = CreateRobot { name };
            let res = self_gd.bind().client
                .post("/robots")
                .body_json(&req)?
                .parse_response_json::<dto::Robot>()
                .await?;
            Ok(SrRobot::create(&res))
        })
    }

    #[func]
    pub fn delete_robot(&self, id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            self_gd.bind().client
                .delete(format!("/robots/{}", id))
                .parse_response()
                .await?;
            Ok(())
        })
    }

    #[func]
    pub fn upload_robot(&self, id: String, code: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            self_gd.bind().client
                .post(format!("/robots/{}/upload", id))
                .body_string(code)
                .parse_response()
                .await?;
            Ok(())
        })
    }

    #[func]
    pub fn download_robot(&self, id: String) -> Gd<SrFuture> {
        self.spawn_result(async move |self_gd| {
            let res = self_gd.bind().client
                .post(format!("/robots/{}/download", id))
                .parse_response()
                .await?;
            Ok(res)
        })
    }

    #[func]
    pub fn set_competing_robot(&self, id: Variant) -> Gd<SrFuture> {
        let competing_robot_id = if id.is_nil() { None } else { Some(id.to_string()) };
        self.spawn_result(async move |mut self_gd| {
            let mut this = self_gd.bind_mut();
            this
                .client
                .post("/users/me/competing-robot")
                .body_json(&UpdateCompetingRobot { competing_robot_id })?
                .parse_response()
                .await?;
            this
                ._refetch_me()
                .await?;
            Ok(())
        })
    }

    #[func]
    pub fn refetch_me(&self) -> Gd<SrFuture> {
        self.spawn_result(async move |mut self_gd| {
            self_gd.bind_mut()._refetch_me().await
        })
    }

    pub async fn _refetch_me(&mut self) -> Result<(), SrClientError> {
        let user = self.client
            .get("/users/me")
            .parse_response_json::<PrivateUser>()
            .await?;
        self.user = Some(user);
        Ok(())
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
    pub fn get_me(&self) -> Option<Gd<SrPrivateUser>> {
        self.user.as_ref().map(SrPrivateUser::create)
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
        self.spawn_gd(async move |self_gd| {
            let result = f(self_gd).await
                .map_err(|err| Gd::from_object(err));
            SrResult::from(result)
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
        } else if let Ok(error) = serde_json::from_str::<dto::Error>(&body) {
            Err(SrClientError::new(&error.error, &error.message))
        } else {
            Err(SrClientError::unknown(&format!("{}: {}", self.status(), body)))
        }
    }
}

impl ParseResponse for surf::RequestBuilder {
    async fn parse_response(self) -> Result<String, SrClientError> {
        self.send().await?.parse_response().await
    }
}
