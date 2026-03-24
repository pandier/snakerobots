use chrono::{DateTime, Utc};
use rowplus_derive::RowPlus;
use snakerobots_shared::dto::{self, MatchRequest, User};
use sqlx::types::Uuid;

use crate::model::PartialUserModel;

#[derive(Debug, Clone, RowPlus)]
#[rowplus(alias = "matches")]
pub struct MatchModel {
    pub id: Uuid,
    pub seed: i64,
    pub winner: Option<Uuid>,
    pub aborted: bool,
    pub played_at: DateTime<Utc>,
}

impl MatchModel {
    pub fn with_players(self, players: Vec<MatchPlayerModel>) -> MatchWithPlayersModel {
        MatchWithPlayersModel { match_: self, players }
    }
}

#[derive(Debug, Clone, RowPlus)]
#[rowplus(alias = "match_players")]
pub struct MatchPlayerModel {
    pub id: i32,
    #[rowplus(nested)]
    pub user: Option<PartialUserModel>,
}

impl From<MatchPlayerModel> for Option<User> {
    fn from(value: MatchPlayerModel) -> Self {
        value.user.map(|user| user.into())
    }
}

#[derive(Debug, Clone)]
pub struct MatchWithPlayersModel {
    pub match_: MatchModel,
    pub players: Vec<MatchPlayerModel>,
}

impl From<MatchWithPlayersModel> for dto::Match {
    fn from(value: MatchWithPlayersModel) -> Self {
        Self {
            id: value.match_.id.to_string(),
            seed: value.match_.seed as u64,
            players: value.players.into_iter().map(|player| player.into()).collect(),
            played_at: value.match_.played_at,
            winner: value.match_.winner.map(|uuid| uuid.to_string()),
        }
    }
}

#[derive(Debug, Clone, RowPlus)]
#[rowplus(alias = "match_requests")]
pub struct MatchRequestModel {
    #[rowplus(nested)]
    pub sender: PartialUserModel,
    #[rowplus(nested)]
    pub receiver: PartialUserModel,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl From<MatchRequestModel> for MatchRequest {
    fn from(value: MatchRequestModel) -> Self {
        Self {
            sender: value.sender.into(),
            receiver: value.receiver.into(),
            created_at: value.created_at,
            expires_at: value.expires_at,
        }
    }
}
