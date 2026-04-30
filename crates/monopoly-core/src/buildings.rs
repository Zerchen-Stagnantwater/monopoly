use crate::board::{Tile, ColorGroup};
use crate::state::GameState;

#[derive(Debug, Clone)]
pub enum BuildResult {
    /// House or hotel successfully built
    Built,
    /// Player can't afford it
    InsufficientFunds,
    /// Player doesn't own the full color group
    IncompleteGroup,
    /// No houses left in the supply
    NoSupply,
    /// Already at max (hotel)
    MaxReached,
    /// Tile is not a property
    NotAProperty,
    /// Player doesn't own this tile
    NotOwner,
}

#[derive(Debug, Clone)]
pub enum MortgageResult {
    Mortgaged { value: u32 },
    Unmortgaged { cost: u32 },
    AlreadyMortgaged,
    NotMortgaged,
    NotOwner,
    InsufficientFunds,
    NotAProperty,
    HasBuildings,
}

/// Build a house or hotel on a property.
pub fn build_house(
    state: &mut GameState,
    tile_index: usize,
    _max_houses: u8,
    _max_hotels: u8,
    houses_remaining: &mut u8,
    hotels_remaining: &mut u8,
) -> BuildResult {
    let player_id = state.players[state.current_player_index].id;

    // Validate ownership and get building cost + color group
    let (building_cost, color_group, houses) = match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner != Some(player_id) { return BuildResult::NotOwner; }
            if p.mortgaged { return BuildResult::NotAProperty; }
            if p.houses >= 5 { return BuildResult::MaxReached; }
            (p.building_cost, p.color_group.clone(), p.houses)
        }
        _ => return BuildResult::NotAProperty,
    };

    // Must own full color group
    if !owns_full_group(state, tile_index, &color_group) {
        return BuildResult::IncompleteGroup;
    }

    // Must build evenly — no property in the group can have more than 1 house
    // fewer than the one being built on
    if !can_build_evenly(state, tile_index, &color_group) {
        return BuildResult::IncompleteGroup;
    }

    // Check supply
    if houses < 4 {
        // Building a house
        if *houses_remaining == 0 { return BuildResult::NoSupply; }
    } else {
        // Building a hotel (houses == 4)
        if *hotels_remaining == 0 { return BuildResult::NoSupply; }
    }

    // Check funds
    if state.players[state.current_player_index].money < building_cost {
        return BuildResult::InsufficientFunds;
    }

    // Deduct cost
    state.players[state.current_player_index].money -= building_cost;

    // Update supply
    if houses < 4 {
        *houses_remaining -= 1;
    } else {
        // Converting 4 houses to a hotel — return 4 houses to supply
        *houses_remaining += 4;
        *hotels_remaining -= 1;
    }

    // Build
    if let Tile::Property(p) = &mut state.board.tiles[tile_index] {
        p.houses += 1;
    }

    BuildResult::Built
}

/// Sell a house or hotel back to the bank (half price).
pub fn sell_house(
    state: &mut GameState,
    tile_index: usize,
    houses_remaining: &mut u8,
    hotels_remaining: &mut u8,
) -> BuildResult {
    let player_id = state.players[state.current_player_index].id;

    let (building_cost, color_group, houses) = match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner != Some(player_id) { return BuildResult::NotOwner; }
            if p.houses == 0 { return BuildResult::NotAProperty; }
            (p.building_cost, p.color_group.clone(), p.houses)
        }
        _ => return BuildResult::NotAProperty,
    };

    // Must sell evenly
    if !can_sell_evenly(state, tile_index, &color_group) {
        return BuildResult::IncompleteGroup;
    }

    let sell_price = building_cost / 2;
    state.players[state.current_player_index].money += sell_price;

    // Update supply
    if houses == 5 {
        // Selling a hotel — returns hotel, gets 4 houses back
        *hotels_remaining += 1;
        *houses_remaining -= 4;
    } else {
        *houses_remaining += 1;
    }

    if let Tile::Property(p) = &mut state.board.tiles[tile_index] {
        p.houses -= 1;
    }

    BuildResult::Built
}

/// Mortgage a property — receive half its value, no rent can be collected.
pub fn mortgage_property(state: &mut GameState, tile_index: usize) -> MortgageResult {
    let player_id = state.players[state.current_player_index].id;

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner != Some(player_id) { return MortgageResult::NotOwner; }
            if p.mortgaged { return MortgageResult::AlreadyMortgaged; }
            if p.houses > 0 { return MortgageResult::HasBuildings; }
        }
        Tile::Railroad(r) => {
            if r.owner != Some(player_id) { return MortgageResult::NotOwner; }
            if r.mortgaged { return MortgageResult::AlreadyMortgaged; }
        }
        Tile::Utility(u) => {
            if u.owner != Some(player_id) { return MortgageResult::NotOwner; }
            if u.mortgaged { return MortgageResult::AlreadyMortgaged; }
        }
        _ => return MortgageResult::NotAProperty,
    }

    let mortgage_value = get_price(&state.board.tiles[tile_index]) / 2;
    state.players[state.current_player_index].money += mortgage_value;

    match &mut state.board.tiles[tile_index] {
        Tile::Property(p) => p.mortgaged = true,
        Tile::Railroad(r) => r.mortgaged = true,
        Tile::Utility(u)  => u.mortgaged = true,
        _ => {}
    }

    MortgageResult::Mortgaged { value: mortgage_value }
}

/// Unmortgage a property — pay back mortgage value plus 10% interest.
pub fn unmortgage_property(state: &mut GameState, tile_index: usize) -> MortgageResult {
    let player_id = state.players[state.current_player_index].id;

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner != Some(player_id) { return MortgageResult::NotOwner; }
            if !p.mortgaged { return MortgageResult::NotMortgaged; }
        }
        Tile::Railroad(r) => {
            if r.owner != Some(player_id) { return MortgageResult::NotOwner; }
            if !r.mortgaged { return MortgageResult::NotMortgaged; }
        }
        Tile::Utility(u) => {
            if u.owner != Some(player_id) { return MortgageResult::NotOwner; }
            if !u.mortgaged { return MortgageResult::NotMortgaged; }
        }
        _ => return MortgageResult::NotAProperty,
    }

    let price = get_price(&state.board.tiles[tile_index]);
    let unmortgage_cost = (price / 2) + (price / 10); // mortgage value + 10%

    if state.players[state.current_player_index].money < unmortgage_cost {
        return MortgageResult::InsufficientFunds;
    }

    state.players[state.current_player_index].money -= unmortgage_cost;

    match &mut state.board.tiles[tile_index] {
        Tile::Property(p) => p.mortgaged = false,
        Tile::Railroad(r) => r.mortgaged = false,
        Tile::Utility(u)  => u.mortgaged = false,
        _ => {}
    }

    MortgageResult::Unmortgaged { cost: unmortgage_cost }
}

// --- Helpers ---

fn get_price(tile: &Tile) -> u32 {
    match tile {
        Tile::Property(p) => p.price,
        Tile::Railroad(r) => r.price,
        Tile::Utility(u)  => u.price,
        _ => 0,
    }
}

fn owns_full_group(state: &GameState, tile_index: usize, group: &ColorGroup) -> bool {
    let owner = match &state.board.tiles[tile_index] {
        Tile::Property(p) => match p.owner {
            Some(o) => o,
            None => return false,
        },
        _ => return false,
    };

    state.board.tiles.iter().all(|t| match t {
        Tile::Property(p) if &p.color_group == group => p.owner == Some(owner),
        _ => true,
    })
}

/// Building must be placed on the property with the fewest houses in the group.
fn can_build_evenly(state: &GameState, tile_index: usize, group: &ColorGroup) -> bool {
    let target_houses = match &state.board.tiles[tile_index] {
        Tile::Property(p) => p.houses,
        _ => return false,
    };

    state.board.tiles.iter().all(|t| match t {
        Tile::Property(p) if &p.color_group == group => p.houses >= target_houses,
        _ => true,
    })
}

/// Selling must come from the property with the most houses in the group.
fn can_sell_evenly(state: &GameState, tile_index: usize, group: &ColorGroup) -> bool {
    let target_houses = match &state.board.tiles[tile_index] {
        Tile::Property(p) => p.houses,
        _ => return false,
    };

    state.board.tiles.iter().all(|t| match t {
        Tile::Property(p) if &p.color_group == group => p.houses <= target_houses,
        _ => true,
    })
}
