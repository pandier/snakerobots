use redis::{AsyncTypedCommands, FromRedisValue, ToRedisArgs, aio::MultiplexedConnection};
use serde::{Deserialize, Serialize};
use snakerobots_shared::{Direction, GameResult};

use crate::error::Result;

const GAMES_KEY: &str = "snakerobots:games";
const GAMES_QUEUE_KEY: &str = "snakerobots:games:queue";
const GAMES_FINISHED_KEY: &str = "snakerobots:games:finished";
const GAME_KEY_PREFIX: &str = "snakerobots:game:";

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingGameId(String);

impl std::fmt::Display for PendingGameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromRedisValue for PendingGameId {
    fn from_redis_value(v: redis::Value) -> std::result::Result<Self, redis::ParsingError> {
        String::from_redis_value(v).map(|s| PendingGameId(s))
    }
}

impl ToRedisArgs for PendingGameId {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + redis::RedisWrite {
        self.0.write_redis_args(out)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingGame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    pub players: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishedGame {
    pub id: PendingGameId,
    pub seed: u64,
    pub players: Vec<FinishedGamePlayer>,
    pub result: GameResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FinishedGamePlayer {
    pub id: String,
    #[serde(with = "snakerobots_shared::dto::util::directions")]
    pub moves: Vec<Direction>,
}

pub struct RedisGameProcessing {
    con: MultiplexedConnection,
}

impl RedisGameProcessing {
    pub fn new(con: MultiplexedConnection) -> Self {
        Self { con }
    }

    pub async fn enqueue(&self, id: &PendingGameId, game: &PendingGame) -> Result<()> {
        let mut con = self.con.clone();

        let json = serde_json::to_string(game)?;

        con.set(game_key(id), json).await?;
        con.sadd(GAMES_KEY, id).await?;
        con.rpush(GAMES_QUEUE_KEY, id).await?;

        // TODO: notify

        Ok(())
    }

    pub async fn poll_queue(&self) -> Result<Option<(PendingGameId, PendingGame)>> {
        let mut con = self.con.clone();

        while let Some(id) = con.lpop(GAMES_QUEUE_KEY, None).await? {
            if let Some(json) = con.get(game_key(&id)).await? {
                let game = serde_json::from_str(&json)?;
                return Ok(Some((id, game)));
            }
        }

        Ok(None)
    }

    pub async fn finish(&self, game: &FinishedGame) -> Result<bool> {
        let mut con = self.con.clone();

        if !con.exists(game_key(&game.id)).await? {
            return Ok(false);
        }

        let json = serde_json::to_string(game)?;
        con.rpush(GAMES_FINISHED_KEY, json).await?;

        // TODO: notify

        Ok(true)
    }

    pub async fn poll_finished(&self) -> Result<Option<FinishedGame>> {
        let mut con = self.con.clone();

        let json: Option<String> = con.lpop(GAMES_FINISHED_KEY, None).await?;
        while let Some(json) = json {
            let game: FinishedGame = serde_json::from_str(&json)?;

            con.del(game_key(&game.id)).await?;
            con.srem(GAMES_KEY, &game.id).await?;

            return Ok(Some(game));
        }

        Ok(None)
    }
}

fn game_key(id: &PendingGameId) -> String {
    format!("{}{}", GAME_KEY_PREFIX, id)
}
