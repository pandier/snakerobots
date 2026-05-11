use godot::prelude::*;
use snakerobots_shared::{dto::{DefaultGameReplay, LeaderboardUser, Match, MatchPlayer, MatchRequest, PrivateUser, Robot, ShortUser, User}, logic::robot::error::InfallibleRobotErrorHandler};

use crate::game::timeline::GameTimeline;

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrUser {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
    #[var]
    pub created_at: i64,
    #[var]
    pub elo: f64,
}

#[godot_api]
impl SrUser {
    pub fn create(user: &User) -> Gd<Self> {
        Gd::from_object(Self {
            id: user.id.to_godot(),
            username: user.username.to_godot(),
            created_at: user.created_at.timestamp(),
            elo: user.elo,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrShortUser {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
}

#[godot_api]
impl SrShortUser {
    pub fn create(user: &ShortUser) -> Gd<Self> {
        Gd::from_object(Self {
            id: user.id.to_godot(),
            username: user.username.to_godot(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrPrivateUser {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
    #[var]
    pub created_at: i64,
    #[var]
    pub elo: f64,
    #[var]
    pub competing_robot_id: Variant,
}

#[godot_api]
impl SrPrivateUser {
    pub fn create(user: &PrivateUser) -> Gd<Self> {
        Gd::from_object(Self {
            id: user.id.to_godot(),
            username: user.username.to_godot(),
            created_at: user.created_at.timestamp(),
            elo: user.elo,
            competing_robot_id: user.competing_robot_id.as_ref()
                .map(|x| x.to_variant())
                .unwrap_or_else(Variant::nil),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrLeaderboardUser {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
    #[var]
    pub elo: f64,
    #[var]
    pub rank: i64,
}

#[godot_api]
impl SrLeaderboardUser {
    pub fn create(user: LeaderboardUser) -> Gd<Self> {
        Gd::from_object(Self {
            id: user.id.to_godot(),
            username: user.username.to_godot(),
            elo: user.elo,
            rank: user.rank,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatch {
    #[var]
    pub id: GString,
    #[var]
    pub winner_id: Variant,
    #[var]
    pub played_at: i64,
    #[var]
    pub players: Array<Option<Gd<SrMatchPlayer>>>,
    #[var]
    pub ranked: bool,
}

#[godot_api]
impl SrMatch {
    pub fn create(match_: Match) -> Gd<Self> {
        Gd::from_object(Self {
            id: match_.id.to_godot(),
            winner_id: match_.winner.as_ref()
                .map(|id| id.to_variant())
                .unwrap_or_else(Variant::nil),
            played_at: match_.played_at.timestamp(),
            players: match_.players.iter()
                .map(|player| player.as_ref()
                .map(SrMatchPlayer::create))
                .collect(),
            ranked: match_.ranked,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatchPlayer {
    #[var]
    pub id: GString,
    #[var]
    pub username: GString,
    #[var]
    pub elo: Variant,
    #[var]
    pub elo_diff: Variant,
}

#[godot_api]
impl SrMatchPlayer {
    pub fn create(value: &MatchPlayer) -> Gd<Self> {
        Gd::from_object(Self {
            id: value.id.to_godot(),
            username: value.username.to_godot(),
            elo: value.elo.as_ref()
                .map(|elo| elo.value.to_variant())
                .unwrap_or_else(Variant::nil),
            elo_diff: value.elo.as_ref()
                .map(|elo| elo.diff.to_variant())
                .unwrap_or_else(Variant::nil),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatchReplay {
    replay: DefaultGameReplay,
}

#[godot_api]
impl SrMatchReplay {
    pub fn create(replay: DefaultGameReplay) -> Gd<Self> {
        Gd::from_object(Self {
            replay
        })
    }

    #[func]
    pub fn create_timeline(&self) -> Gd<GameTimeline> {
        let game = self.replay.create_game();
        let timeline = GameTimeline::evaluate::<InfallibleRobotErrorHandler>(game)
            .expect("infallible");
        Gd::from_object(timeline)
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatchRequest {
    #[var]
    pub receiver: Gd<SrShortUser>,
    #[var]
    pub sender: Gd<SrShortUser>,
    #[var]
    pub created_at: i64,
    #[var]
    pub expires_at: i64,
}

#[godot_api]
impl SrMatchRequest {
    pub fn create(req: &MatchRequest) -> Gd<Self> {
        Gd::from_object(Self {
            receiver: SrShortUser::create(&req.receiver),
            sender: SrShortUser::create(&req.sender),
            created_at: req.created_at.timestamp(),
            expires_at: req.expires_at.timestamp(),
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrRobot {
    #[var]
    pub id: GString,
    #[var]
    pub name: GString,
    #[var]
    pub created_at: i64,
    #[var]
    pub edited_at: i64,
}

#[godot_api]
impl SrRobot {
    pub fn create(req: &Robot) -> Gd<Self> {
        Gd::from_object(Self {
            id: req.id.to_godot(),
            name: req.name.to_godot(),
            created_at: req.created_at.timestamp(),
            edited_at: req.edited_at.timestamp(),
        })
    }
}
