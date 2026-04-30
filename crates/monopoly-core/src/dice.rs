use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceRoll {
    pub die1: u8,
    pub die2: u8,
}

impl DiceRoll {
    pub fn roll() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            die1: rng.gen_range(1..=6),
            die2: rng.gen_range(1..=6),
        }
    }

    pub fn total(&self) -> u8 {
        self.die1 + self.die2
    }

    pub fn is_doubles(&self) -> bool {
        self.die1 == self.die2
    }
}
