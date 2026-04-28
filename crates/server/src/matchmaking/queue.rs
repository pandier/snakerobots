use rand::{RngExt, SeedableRng, prelude::SliceRandom, rngs::Xoshiro256PlusPlus};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MatchmakingEntry {
    pub id: Uuid,
    pub elo: i32,
    pub robot_id: Uuid,
    matched: bool,
}

impl MatchmakingEntry {
    pub fn new(id: Uuid, elo: i32, robot_id: Uuid) -> Self {
        Self {
            id,
            elo,
            robot_id,
            matched: false,
        }
    }
}

impl Eq for MatchmakingEntry {}

impl PartialEq for MatchmakingEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

pub struct MatchmakingQueue {
    rng: Xoshiro256PlusPlus,
    ranks: Vec<MatchmakingEntry>,
    queue: Vec<usize>,
}

impl MatchmakingQueue {

    pub fn new(mut ranks: Vec<MatchmakingEntry>) -> Self {
        ranks.sort_by_key(|x| x.elo);

        let mut queue = (0..ranks.len()).collect::<Vec<_>>();

        let mut rng = Xoshiro256PlusPlus::from_rng(&mut rand::rng());

        queue.shuffle(&mut rng);

        Self {
            rng,
            ranks,
            queue,
        }
    }

    pub fn next_match(&mut self) -> Option<(&MatchmakingEntry, &MatchmakingEntry)> {
        while let Some(index) = self.queue.pop() {
            if self.ranks[index].matched {
                continue;
            }

            let opponents = (index.saturating_sub(5)..index.saturating_add(6).min(self.ranks.len()))
                .filter(|j| *j != index)
                .filter(|j| {
                    let e = &self.ranks[*j];
                    !e.matched
                })
                .collect::<Vec<_>>();

            if opponents.is_empty() {
                continue;
            }

            let o_index = opponents[self.rng.random_range(0..opponents.len())];
            
            self.ranks[index].matched = true;
            self.ranks[o_index].matched = true;

            return Some((&self.ranks[index], &self.ranks[o_index]));
        }

        None
    }
}
