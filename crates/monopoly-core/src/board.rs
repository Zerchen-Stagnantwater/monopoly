use serde::{Deserialize, Serialize};

/// Every tile on the board is one of these variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Tile {
    /// A buyable property (dark purple, light blue, etc.)
    Property(PropertyTile),
    /// One of the four railroads
    Railroad(RailroadTile),
    /// One of the two utilities
    Utility(UtilityTile),
    /// Draw a Community Chest card
    CommunityChest,
    /// Draw a Chance card
    Chance,
    /// Pay a flat tax
    Tax(TaxTile),
    /// Start tile — collect salary when passing
    Go,
    /// Just visiting / in jail
    Jail,
    /// Go to jail immediately
    GoToJail,
    /// Free parking — no effect in standard rules
    FreeParking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyTile {
    pub name: String,
    pub color_group: ColorGroup,
    pub price: u32,
    pub building_cost: u32,
    /// Rent at 0, 1, 2, 3, 4 houses, then hotel
    pub rent: [u32; 6],
    pub owner: Option<u8>, // player index
    pub houses: u8,        // 0–4 houses, 5 = hotel
    pub mortgaged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RailroadTile {
    pub name: String,
    pub price: u32,
    pub owner: Option<u8>,
    pub mortgaged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityTile {
    pub name: String,
    pub price: u32,
    pub owner: Option<u8>,
    pub mortgaged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxTile {
    pub name: String,
    pub amount: u32,
}

/// The eight color groups on a standard Monopoly board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorGroup {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    DarkBlue,
}

/// The board itself — 40 tiles in order starting from Go.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub tiles: Vec<Tile>,
}

impl Board {
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Wraps position around the board (always 0–39).
    pub fn normalize_position(&self, pos: usize) -> usize {
        pos % self.tiles.len()
    }
}
