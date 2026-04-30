use crate::board::Tile;
use crate::state::GameState;

#[derive(Debug, Clone)]
pub struct TradeOffer {
    pub from_player: u8,
    pub to_player: u8,
    pub offered_properties: Vec<usize>,  // tile indices
    pub offered_money: u32,
    pub requested_properties: Vec<usize>, // tile indices
    pub requested_money: u32,
}

#[derive(Debug, Clone)]
pub enum TradeResult {
    Completed,
    InsufficientFunds { player: u8 },
    PropertyNotOwned { tile_index: usize, player: u8 },
    HasBuildings { tile_index: usize },
    InvalidPlayer,
}

/// Validate and execute a trade between two players.
pub fn execute_trade(state: &mut GameState, offer: TradeOffer) -> TradeResult {
    // Validate both players exist
    let from_index = match state.players.iter().position(|p| p.id == offer.from_player) {
        Some(i) => i,
        None => return TradeResult::InvalidPlayer,
    };
    let to_index = match state.players.iter().position(|p| p.id == offer.to_player) {
        Some(i) => i,
        None => return TradeResult::InvalidPlayer,
    };

    // Validate offered properties belong to from_player and have no buildings
    for &tile_index in &offer.offered_properties {
        match &state.board.tiles[tile_index] {
            Tile::Property(p) => {
                if p.owner != Some(offer.from_player) {
                    return TradeResult::PropertyNotOwned { tile_index, player: offer.from_player };
                }
                if p.houses > 0 {
                    return TradeResult::HasBuildings { tile_index };
                }
            }
            Tile::Railroad(r) => {
                if r.owner != Some(offer.from_player) {
                    return TradeResult::PropertyNotOwned { tile_index, player: offer.from_player };
                }
            }
            Tile::Utility(u) => {
                if u.owner != Some(offer.from_player) {
                    return TradeResult::PropertyNotOwned { tile_index, player: offer.from_player };
                }
            }
            _ => return TradeResult::PropertyNotOwned { tile_index, player: offer.from_player },
        }
    }

    // Validate requested properties belong to to_player and have no buildings
    for &tile_index in &offer.requested_properties {
        match &state.board.tiles[tile_index] {
            Tile::Property(p) => {
                if p.owner != Some(offer.to_player) {
                    return TradeResult::PropertyNotOwned { tile_index, player: offer.to_player };
                }
                if p.houses > 0 {
                    return TradeResult::HasBuildings { tile_index };
                }
            }
            Tile::Railroad(r) => {
                if r.owner != Some(offer.to_player) {
                    return TradeResult::PropertyNotOwned { tile_index, player: offer.to_player };
                }
            }
            Tile::Utility(u) => {
                if u.owner != Some(offer.to_player) {
                    return TradeResult::PropertyNotOwned { tile_index, player: offer.to_player };
                }
            }
            _ => return TradeResult::PropertyNotOwned { tile_index, player: offer.to_player },
        }
    }

    // Validate funds
    if state.players[from_index].money < offer.offered_money {
        return TradeResult::InsufficientFunds { player: offer.from_player };
    }
    if state.players[to_index].money < offer.requested_money {
        return TradeResult::InsufficientFunds { player: offer.to_player };
    }

    // Execute — transfer money
    state.players[from_index].money -= offer.offered_money;
    state.players[to_index].money += offer.offered_money;
    state.players[to_index].money -= offer.requested_money;
    state.players[from_index].money += offer.requested_money;

    // Transfer offered properties from -> to
    for &tile_index in &offer.offered_properties {
        match &mut state.board.tiles[tile_index] {
            Tile::Property(p) => p.owner = Some(offer.to_player),
            Tile::Railroad(r) => r.owner = Some(offer.to_player),
            Tile::Utility(u)  => u.owner = Some(offer.to_player),
            _ => {}
        }
        state.players[from_index].properties.retain(|&t| t != tile_index);
        state.players[to_index].properties.push(tile_index);
    }

    // Transfer requested properties to -> from
    for &tile_index in &offer.requested_properties {
        match &mut state.board.tiles[tile_index] {
            Tile::Property(p) => p.owner = Some(offer.from_player),
            Tile::Railroad(r) => r.owner = Some(offer.from_player),
            Tile::Utility(u)  => u.owner = Some(offer.from_player),
            _ => {}
        }
        state.players[to_index].properties.retain(|&t| t != tile_index);
        state.players[from_index].properties.push(tile_index);
    }

    TradeResult::Completed
}
