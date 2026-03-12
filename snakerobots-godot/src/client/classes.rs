use godot::prelude::*;
use snakerobots_shared::dto::{MatchRequest, User};

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
