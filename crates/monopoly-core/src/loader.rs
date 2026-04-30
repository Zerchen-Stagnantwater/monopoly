use anyhow::{Context, Result};
use serde::Deserialize;
use crate::board::{Board, Tile, PropertyTile, RailroadTile, UtilityTile, TaxTile, ColorGroup};

/// Raw TOML representation of a tile — deserialized directly from config.
#[derive(Debug, Deserialize)]
struct RawTile {
    #[serde(rename = "type")]
    tile_type: String,
    name: Option<String>,
    color_group: Option<String>,
    price: Option<u32>,
    building_cost: Option<u32>,
    rent: Option<[u32; 6]>,
    amount: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RawBoard {
    tiles: Vec<RawTile>,
}

/// Load a board from a TOML config file path.
pub fn load_board(path: &str) -> Result<Board> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read board config: {}", path))?;

    let raw: RawBoard = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse board config: {}", path))?;

    let tiles = raw.tiles
        .into_iter()
        .enumerate()
        .map(|(i, t)| parse_tile(t, i))
        .collect::<Result<Vec<Tile>>>()?;

    anyhow::ensure!(
        tiles.len() == 40,
        "Board must have exactly 40 tiles, found {}",
        tiles.len()
    );

    Ok(Board { tiles })
}

fn parse_tile(raw: RawTile, index: usize) -> Result<Tile> {
    match raw.tile_type.as_str() {
        "Go"          => Ok(Tile::Go),
        "Jail"        => Ok(Tile::Jail),
        "FreeParking" => Ok(Tile::FreeParking),
        "GoToJail"    => Ok(Tile::GoToJail),
        "CommunityChest" => Ok(Tile::CommunityChest),
        "Chance"      => Ok(Tile::Chance),

        "Tax" => Ok(Tile::Tax(TaxTile {
            name: raw.name.with_context(|| format!("Tile {}: Tax missing name", index))?,
            amount: raw.amount.with_context(|| format!("Tile {}: Tax missing amount", index))?,
        })),

        "Railroad" => Ok(Tile::Railroad(RailroadTile {
            name: raw.name.with_context(|| format!("Tile {}: Railroad missing name", index))?,
            price: raw.price.with_context(|| format!("Tile {}: Railroad missing price", index))?,
            owner: None,
            mortgaged: false,
        })),

        "Utility" => Ok(Tile::Utility(UtilityTile {
            name: raw.name.with_context(|| format!("Tile {}: Utility missing name", index))?,
            price: raw.price.with_context(|| format!("Tile {}: Utility missing price", index))?,
            owner: None,
            mortgaged: false,
        })),

        "Property" => Ok(Tile::Property(PropertyTile {
            name: raw.name.with_context(|| format!("Tile {}: Property missing name", index))?,
            color_group: parse_color(
                &raw.color_group.with_context(|| format!("Tile {}: Property missing color_group", index))?,
                index,
            )?,
            price: raw.price.with_context(|| format!("Tile {}: Property missing price", index))?,
            building_cost: raw.building_cost.with_context(|| format!("Tile {}: Property missing building_cost", index))?,
            rent: raw.rent.with_context(|| format!("Tile {}: Property missing rent", index))?,
            owner: None,
            houses: 0,
            mortgaged: false,
        })),

        other => anyhow::bail!("Tile {}: Unknown tile type '{}'", index, other),
    }
}

fn parse_color(s: &str, index: usize) -> Result<ColorGroup> {
    match s {
        "Brown"    => Ok(ColorGroup::Brown),
        "LightBlue"=> Ok(ColorGroup::LightBlue),
        "Pink"     => Ok(ColorGroup::Pink),
        "Orange"   => Ok(ColorGroup::Orange),
        "Red"      => Ok(ColorGroup::Red),
        "Yellow"   => Ok(ColorGroup::Yellow),
        "Green"    => Ok(ColorGroup::Green),
        "DarkBlue" => Ok(ColorGroup::DarkBlue),
        other => anyhow::bail!("Tile {}: Unknown color group '{}'", index, other),
    }
}
