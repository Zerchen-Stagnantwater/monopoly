use crate::board::Tile;
use crate::state::GameState;

#[derive(Debug, Clone)]
pub enum BankruptcyResult {
    /// Player successfully declared bankrupt, assets transferred
    Bankrupt { eliminated: u8 },
    /// Only one player remains — they win
    GameOver { winner: u8 },
    /// Player is not actually insolvent
    NotInsolvent,
}

/// Total liquidation value a player can raise —
/// current cash + half value of all unmortgaged properties + half value of buildings.
pub fn liquidation_value(state: &GameState, player_id: u8) -> u32 {
    let player = match state.players.iter().find(|p| p.id == player_id) {
        Some(p) => p,
        None => return 0,
    };

    let mut total = player.money;

    for &tile_index in &player.properties {
        match &state.board.tiles[tile_index] {
            Tile::Property(p) => {
                if !p.mortgaged {
                    total += p.price / 2;
                }
                // Sell back buildings at half price
                if p.houses > 0 && p.houses < 5 {
                    total += (p.building_cost / 2) * p.houses as u32;
                } else if p.houses == 5 {
                    total += (p.building_cost / 2) * 4; // hotel = 4 houses worth
                }
            }
            Tile::Railroad(r) => {
                if !r.mortgaged { total += r.price / 2; }
            }
            Tile::Utility(u) => {
                if !u.mortgaged { total += u.price / 2; }
            }
            _ => {}
        }
    }

    total
}

/// Declare a player bankrupt — transfer all assets to creditor (or bank if None).
/// Returns GameOver if only one active player remains.
pub fn declare_bankrupt(
    state: &mut GameState,
    player_id: u8,
    creditor: Option<u8>,
) -> BankruptcyResult {
    // Check player is actually insolvent
    let is_insolvent = state.players.iter()
        .find(|p| p.id == player_id)
        .map(|p| p.money == 0 || p.bankrupt)
        .unwrap_or(false);

    if !is_insolvent {
        return BankruptcyResult::NotInsolvent;
    }

    let bankrupt_index = match state.players.iter().position(|p| p.id == player_id) {
        Some(i) => i,
        None => return BankruptcyResult::NotInsolvent,
    };

    match creditor {
        Some(creditor_id) => {
            // Transfer all money to creditor
            let money = state.players[bankrupt_index].money;
            let creditor_index = state.players.iter().position(|p| p.id == creditor_id).unwrap();

            state.players[creditor_index].money += money;
            state.players[bankrupt_index].money = 0;

            // Transfer all properties to creditor (unmortgaged at no cost)
            let properties = state.players[bankrupt_index].properties.clone();
            for tile_index in properties {
                match &mut state.board.tiles[tile_index] {
                    Tile::Property(p) => {
                        p.owner = Some(creditor_id);
                        p.houses = 0; // buildings are lost
                    }
                    Tile::Railroad(r) => r.owner = Some(creditor_id),
                    Tile::Utility(u)  => u.owner = Some(creditor_id),
                    _ => {}
                }
                state.players[creditor_index].properties.push(tile_index);
            }

            state.players[bankrupt_index].properties.clear();
        }

        None => {
            // Bankrupt to the bank — properties return to unowned
            let properties = state.players[bankrupt_index].properties.clone();
            for tile_index in properties {
                match &mut state.board.tiles[tile_index] {
                    Tile::Property(p) => {
                        p.owner = None;
                        p.houses = 0;
                        p.mortgaged = false;
                    }
                    Tile::Railroad(r) => {
                        r.owner = None;
                        r.mortgaged = false;
                    }
                    Tile::Utility(u) => {
                        u.owner = None;
                        u.mortgaged = false;
                    }
                    _ => {}
                }
            }
            state.players[bankrupt_index].properties.clear();
            state.players[bankrupt_index].money = 0;
        }
    }

    // Mark player as bankrupt
    state.players[bankrupt_index].bankrupt = true;

    // Check win condition
    let active: Vec<u8> = state.players.iter()
        .filter(|p| !p.bankrupt)
        .map(|p| p.id)
        .collect();

    if active.len() == 1 {
        let winner = active[0];
        state.game_over = true;
        state.winner = Some(winner);
        return BankruptcyResult::GameOver { winner };
    }

    BankruptcyResult::Bankrupt { eliminated: player_id }
}

/// Check if a player cannot pay a debt even after full liquidation.
pub fn is_bankrupt(state: &GameState, player_id: u8, debt: u32) -> bool {
    liquidation_value(state, player_id) < debt
}
