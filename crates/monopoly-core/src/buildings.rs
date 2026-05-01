use crate::board::{Tile, ColorGroup};
use crate::state::GameState;

#[derive(Debug, Clone)]
pub enum BuildResult {
    Built,
    InsufficientFunds,
    IncompleteGroup,
    NoSupply,
    MaxReached,
    NotAProperty,
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

pub fn build_house(state: &mut GameState, tile_index: usize) -> BuildResult {
    let player_id = state.players[state.current_player_index].id;

    let (building_cost, color_group, houses) = match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner != Some(player_id) { return BuildResult::NotOwner; }
            if p.mortgaged { return BuildResult::NotAProperty; }
            if p.houses >= 5 { return BuildResult::MaxReached; }
            (p.building_cost, p.color_group.clone(), p.houses)
        }
        _ => return BuildResult::NotAProperty,
    };

    if !owns_full_group(state, tile_index, &color_group) {
        return BuildResult::IncompleteGroup;
    }

    if !can_build_evenly(state, tile_index, &color_group) {
        return BuildResult::IncompleteGroup;
    }

    if houses < 4 {
        if state.houses_remaining == 0 { return BuildResult::NoSupply; }
    } else {
        if state.hotels_remaining == 0 { return BuildResult::NoSupply; }
    }

    if state.players[state.current_player_index].money < building_cost {
        return BuildResult::InsufficientFunds;
    }

    state.players[state.current_player_index].money -= building_cost;

    if houses < 4 {
        state.houses_remaining -= 1;
    } else {
        state.houses_remaining += 4;
        state.hotels_remaining -= 1;
    }

    if let Tile::Property(p) = &mut state.board.tiles[tile_index] {
        p.houses += 1;
    }

    BuildResult::Built
}

pub fn sell_house(state: &mut GameState, tile_index: usize) -> BuildResult {
    let player_id = state.players[state.current_player_index].id;

    let (building_cost, color_group, houses) = match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.owner != Some(player_id) { return BuildResult::NotOwner; }
            if p.houses == 0 { return BuildResult::NotAProperty; }
            (p.building_cost, p.color_group.clone(), p.houses)
        }
        _ => return BuildResult::NotAProperty,
    };

    if !can_sell_evenly(state, tile_index, &color_group) {
        return BuildResult::IncompleteGroup;
    }

    let sell_price = building_cost / 2;
    state.players[state.current_player_index].money += sell_price;

    if houses == 5 {
        state.hotels_remaining += 1;
        state.houses_remaining -= 4;
    } else {
        state.houses_remaining += 1;
    }

    if let Tile::Property(p) = &mut state.board.tiles[tile_index] {
        p.houses -= 1;
    }

    BuildResult::Built
}

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
    let unmortgage_cost = (price / 2) + (price / 10);

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
