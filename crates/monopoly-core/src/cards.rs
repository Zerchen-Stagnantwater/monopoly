use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

/// A single card effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardEffect {
    /// Collect money from the bank
    Collect { amount: u32 },
    /// Pay money to the bank
    Pay { amount: u32 },
    /// Move to a specific tile index
    MoveTo { tile_index: usize },
    /// Move back n spaces
    MoveBack { spaces: usize },
    /// Go to jail immediately
    GoToJail,
    /// Receive a get out of jail free card
    GetOutOfJail,
    /// Collect from each other player
    CollectFromPlayers { amount: u32 },
    /// Pay each other player
    PayPerPlayer { amount: u32 },
    /// Pay per house and hotel owned
    PayPerHouse { per_house: u32, per_hotel: u32 },
    /// Move to nearest utility
    MoveToNearestUtility,
    /// Move to nearest railroad
    MoveToNearestRailroad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub description: String,
    pub effect: CardEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDecks {
    pub community_chest: Vec<Card>,
    pub chance: Vec<Card>,
    pub community_chest_index: usize,
    pub chance_index: usize,
}

impl CardDecks {
    pub fn draw_community_chest(&mut self) -> Card {
        let card = self.community_chest[self.community_chest_index].clone();
        self.community_chest_index =
            (self.community_chest_index + 1) % self.community_chest.len();
        card
    }

    pub fn draw_chance(&mut self) -> Card {
        let card = self.chance[self.chance_index].clone();
        self.chance_index = (self.chance_index + 1) % self.chance.len();
        card
    }
}

// --- TOML raw types ---

#[derive(Debug, Deserialize)]
struct RawCard {
    description: String,
    effect: String,
    value: Option<u32>,
    hotel_value: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RawDecks {
    community_chest: Vec<RawCard>,
    chance: Vec<RawCard>,
}

pub fn load_card_decks(path: &str) -> Result<CardDecks> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read card deck: {}", path))?;

    let raw: RawDecks = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse card deck: {}", path))?;

    let community_chest = raw.community_chest
        .into_iter()
        .map(parse_card)
        .collect::<Result<Vec<Card>>>()?;

    let chance = raw.chance
        .into_iter()
        .map(parse_card)
        .collect::<Result<Vec<Card>>>()?;

    // Shuffle both decks
    let community_chest = shuffle(community_chest);
    let chance = shuffle(chance);

    Ok(CardDecks {
        community_chest,
        chance,
        community_chest_index: 0,
        chance_index: 0,
    })
}

fn parse_card(raw: RawCard) -> Result<Card> {
    let effect = match raw.effect.as_str() {
        "collect" => CardEffect::Collect {
            amount: raw.value.unwrap_or(0),
        },
        "pay" => CardEffect::Pay {
            amount: raw.value.unwrap_or(0),
        },
        "move_to" => CardEffect::MoveTo {
            tile_index: raw.value.unwrap_or(0) as usize,
        },
        "move_back" => CardEffect::MoveBack {
            spaces: raw.value.unwrap_or(0) as usize,
        },
        "go_to_jail"      => CardEffect::GoToJail,
        "get_out_of_jail" => CardEffect::GetOutOfJail,
        "collect_from_players" => CardEffect::CollectFromPlayers {
            amount: raw.value.unwrap_or(0),
        },
        "pay_per_player" => CardEffect::PayPerPlayer {
            amount: raw.value.unwrap_or(0),
        },
        "pay_per_house" => CardEffect::PayPerHouse {
            per_house: raw.value.unwrap_or(0),
            per_hotel: raw.hotel_value.unwrap_or(0),
        },
        "move_to_nearest_utility"  => CardEffect::MoveToNearestUtility,
        "move_to_nearest_railroad" => CardEffect::MoveToNearestRailroad,
        other => anyhow::bail!("Unknown card effect: {}", other),
    };

    Ok(Card {
        description: raw.description,
        effect,
    })
}

fn shuffle(mut cards: Vec<Card>) -> Vec<Card> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize;

    // Fisher-Yates shuffle with a simple LCG
    let n = cards.len();
    let mut rng = seed;
    for i in (1..n).rev() {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = rng % (i + 1);
        cards.swap(i, j);
    }
    cards
}
