use godot::prelude::*;
use snakerobots_shared::dto::{Match, MatchPlayer, MatchRequest, User};

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

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatch {
    match_: Match,
    #[var]
    pub id: GString,
    #[var]
    pub seed: i64,
    #[var]
    pub played_at: i64,
    #[var]
    pub players: Array<Gd<SrMatchPlayer>>,
}

#[godot_api]
impl SrMatch {
    pub fn create(match_: Match) -> Gd<Self> {
        Gd::from_object(Self {
            seed: match_.seed as i64,
            id: match_.id.to_godot(),
            played_at: match_.played_at.timestamp(),
            players: match_.players.iter().map(SrMatchPlayer::create).collect(),
            match_
        })
    }

    // TODO: create_timeline
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatchPlayer {
    #[var]
    pub user_id: Variant,
}

#[godot_api]
impl SrMatchPlayer {
    pub fn create(req: &MatchPlayer) -> Gd<Self> {
        Gd::from_object(Self {
            user_id: req.user_id.as_ref().map(ToGodot::to_variant).unwrap_or_else(Variant::nil)
        })
    }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatchRequest {
    #[var]
    pub receiver_id: GString,
    #[var]
    pub sender_id: GString,
    #[var]
    pub created_at: i64,
    #[var]
    pub expires_at: i64,
}

#[godot_api]
impl SrMatchRequest {
    pub fn create(req: &MatchRequest) -> Gd<Self> {
        Gd::from_object(Self {
            receiver_id: req.receiver_id.to_godot(),
            sender_id: req.sender_id.to_godot(),
            created_at: req.created_at.timestamp(),
            expires_at: req.expires_at.timestamp(),
        })
    }
}
