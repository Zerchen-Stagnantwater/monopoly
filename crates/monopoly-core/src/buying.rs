use crate::board::Tile;
use crate::state::{GameState, TurnPhase};

/// Result of a buy attempt.
#[derive(Debug, Clone)]
pub enum BuyResult {
    /// Property successfully purchased
    Purchased,
    /// Player couldn't afford it
    InsufficientFunds,
    /// Tile is not buyable
    NotBuyable,
    /// Tile is already owned
    AlreadyOwned,
}

/// Current player attempts to buy the tile at tile_index.
pub fn buy_property(state: &mut GameState, tile_index: usize) -> BuyResult {
    let player_id = state.players[state.current_player_index].id;
    let player_money = state.players[state.current_player_index].money;

    let price = match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner.is_some() { return BuyResult::AlreadyOwned; }
            p.price
        }
        Tile::Railroad(r) => {
            if r.owner.is_some() { return BuyResult::AlreadyOwned; }
            r.price
        }
        Tile::Utility(u) => {
            if u.owner.is_some() { return BuyResult::AlreadyOwned; }
            u.price
        }
        _ => return BuyResult::NotBuyable,
    };

    if player_money < price {
        return BuyResult::InsufficientFunds;
    }

    // Deduct money
    state.players[state.current_player_index].money -= price;

    // Assign ownership
    match &mut state.board.tiles[tile_index] {
        Tile::Property(p) => p.owner = Some(player_id),
        Tile::Railroad(r) => r.owner = Some(player_id),
        Tile::Utility(u)  => u.owner = Some(player_id),
        _ => {}
    }

    // Track in player's property list
    state.players[state.current_player_index].properties.push(tile_index);

    state.turn_phase = TurnPhase::PostRoll;
    BuyResult::Purchased
}

/// Decline to buy — trigger an auction if rules allow, otherwise just move on.
pub fn decline_purchase(state: &mut GameState, tile_index: usize, auction_enabled: bool) {
    if auction_enabled {
        state.turn_phase = TurnPhase::Auction {
            tile_index,
            highest_bid: 0,
            highest_bidder: state.players[state.current_player_index].id,
        };
    } else {
        state.turn_phase = TurnPhase::PostRoll;
    }
}

/// Submit an auction bid for the current auction.
pub fn place_bid(state: &mut GameState, player_id: u8, bid: u32) -> bool {
    if let TurnPhase::Auction { highest_bid, highest_bidder, .. } = &mut state.turn_phase {
        if bid > *highest_bid {
            // Check player can afford it
            let can_afford = state.players.iter()
                .find(|p| p.id == player_id)
                .map(|p| p.money >= bid)
                .unwrap_or(false);

            if can_afford {
                *highest_bid = bid;
                *highest_bidder = player_id;
                return true;
            }
        }
    }
    false
}

/// Finalize the auction — award property to highest bidder.
pub fn finalize_auction(state: &mut GameState) {
    let (tile_index, highest_bid, highest_bidder) = match state.turn_phase {
        TurnPhase::Auction { tile_index, highest_bid, highest_bidder } => {
            (tile_index, highest_bid, highest_bidder)
        }
        _ => return,
    };

    // Only complete if there was at least one real bid
    if highest_bid == 0 {
        state.turn_phase = TurnPhase::PostRoll;
        return;
    }

    // Find bidder index
    let bidder_index = match state.players.iter().position(|p| p.id == highest_bidder) {
        Some(i) => i,
        None => return,
    };

    state.players[bidder_index].money -= highest_bid;

    match &mut state.board.tiles[tile_index] {
        Tile::Property(p) => p.owner = Some(highest_bidder),
        Tile::Railroad(r) => r.owner = Some(highest_bidder),
        Tile::Utility(u)  => u.owner = Some(highest_bidder),
        _ => {}
    }

    state.players[bidder_index].properties.push(tile_index);
    state.turn_phase = TurnPhase::PostRoll;
}
