use godot::prelude::*;
use snakerobots_shared::dto::{Match, MatchRequest, User};

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
    pub players: Array<Option<Gd<SrUser>>>,
}

#[godot_api]
impl SrMatch {
    pub fn create(match_: Match) -> Gd<Self> {
        Gd::from_object(Self {
            seed: match_.seed as i64,
            id: match_.id.to_godot(),
            played_at: match_.played_at.timestamp(),
            players: match_.players.iter().map(|player| player.as_ref().map(SrUser::create)).collect(),
            match_
        })
    }

    // TODO: asynchronous
    // TODO:
    // #[func]
    // pub fn create_timeline(&self) -> Gd<GameTimeline> {
    //     let players = logic::standard::create_standard_snakes()
    //         .into_iter()
    //         .enumerate()
    //         .map(|(i, snake)| {
    //             let robot = ReplayRobot::new(self.match_.players[i].moves.clone());
    //             Player::new(snake, Box::new(robot))
    //         })
    //         .collect();
    //     let size = Size::new(STANDARD_WIDTH, STANDARD_HEIGHT);
    //     let game = Game::new(size, STANDARD_APPLE_COUNT, self.match_.seed, players)
    //         .expect("invalid game layout");
    //     let timeline = GameTimeline::run_game(game);
    //     Gd::from_object(timeline)
    // }
}

#[derive(GodotClass)]
#[class(no_init, base=RefCounted)]
pub struct SrMatchRequest {
    #[var]
    pub receiver: Gd<SrUser>,
    #[var]
    pub sender: Gd<SrUser>,
    #[var]
    pub created_at: i64,
    #[var]
    pub expires_at: i64,
}

#[godot_api]
impl SrMatchRequest {
    pub fn create(req: &MatchRequest) -> Gd<Self> {
        Gd::from_object(Self {
            receiver: SrUser::create(&req.receiver),
            sender: SrUser::create(&req.sender),
            created_at: req.created_at.timestamp(),
            expires_at: req.expires_at.timestamp(),
        })
    }
}
