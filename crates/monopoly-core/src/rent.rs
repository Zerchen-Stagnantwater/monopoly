use crate::board::{Tile, ColorGroup};
use crate::state::GameState;
use crate::dice::DiceRoll;

/// Calculate the rent owed when landing on an owned tile.
pub fn calculate_rent(state: &GameState, tile_index: usize, roll: &DiceRoll) -> u32 {
    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            if p.mortgaged {
                return 0;
            }
            let base_rent = p.rent[p.houses as usize];
            // Double rent if owner has full color group and no houses built
            if p.houses == 0 && owns_full_group(state, tile_index, &p.color_group) {
                base_rent * 2
            } else {
                base_rent
            }
        }

        Tile::Railroad(r) => {
            if r.mortgaged {
                return 0;
            }
            let owned = count_owned_railroads(state, r.owner.unwrap());
            match owned {
                1 => 25,
                2 => 50,
                3 => 100,
                4 => 200,
                _ => 0,
            }
        }

        Tile::Utility(u) => {
            if u.mortgaged {
                return 0;
            }
            let owned = count_owned_utilities(state, u.owner.unwrap());
            let dice_total = roll.total() as u32;
            match owned {
                1 => dice_total * 4,
                2 => dice_total * 10,
                _ => 0,
            }
        }

        _ => 0,
    }
}

/// Check if the owner of a tile owns all properties in that color group.
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

fn count_owned_railroads(state: &GameState, owner: u8) -> u32 {
    state.board.tiles.iter().filter(|t| matches!(t, Tile::Railroad(r) if r.owner == Some(owner))).count() as u32
}

fn count_owned_utilities(state: &GameState, owner: u8) -> u32 {
    state.board.tiles.iter().filter(|t| matches!(t, Tile::Utility(u) if u.owner == Some(owner))).count() as u32
}
