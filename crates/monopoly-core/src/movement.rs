use crate::{GameState, DiceRoll};
use crate::state::TurnPhase;

/// Result of moving a player — tells the engine what to do next.
#[derive(Debug, Clone)]
pub enum MoveResult {
    /// Player landed on an unowned buyable tile
    LandedOnProperty { tile_index: usize },
    /// Player landed on a tile owned by another player
    LandedOnOwnedProperty { tile_index: usize, owner: u8 },
    /// Player landed on or passed Go
    PassedGo,
    /// Player was sent to jail
    SentToJail,
    /// Player landed on a tax tile
    LandedOnTax { amount: u32 },
    /// Player landed on Community Chest
    LandedOnCommunityChest { tile_index: usize },
    /// Player landed on Chance
    LandedOnChance { tile_index: usize },
    /// Nothing special — Free Parking, Jail (just visiting), own property
    NoEffect,
}

/// Advance the current player by the dice roll, handle Go, handle GoToJail.
/// Returns what the engine needs to act on next.
pub fn advance_player(state: &mut GameState, roll: &DiceRoll, go_salary: u32) -> MoveResult {
    let board_size = state.board.tile_count();
    let player = &state.players[state.current_player_index];
    let old_pos = player.position;
    let new_pos = (old_pos + roll.total() as usize) % board_size;

    // Check if player passed or landed on Go
    let passed_go = new_pos < old_pos || new_pos == 0;

    // Update position
    state.players[state.current_player_index].position = new_pos;

    // Pay Go salary before anything else
    if passed_go {
        state.players[state.current_player_index].money += go_salary;
    }

    // Record the roll
    state.last_roll = Some((roll.die1, roll.die2));
    state.turn_phase = TurnPhase::PostRoll;

    // Check what tile we landed on
    resolve_landing(state, new_pos, passed_go)
}

fn resolve_landing(state: &mut GameState, tile_index: usize, passed_go: bool) -> MoveResult {
    use crate::board::Tile;

    match &state.board.tiles[tile_index] {
        Tile::Go => MoveResult::PassedGo,

        Tile::GoToJail => {
            send_to_jail(state);
            MoveResult::SentToJail
        }

        Tile::Tax(t) => MoveResult::LandedOnTax { amount: t.amount },

        Tile::CommunityChest => MoveResult::LandedOnCommunityChest { tile_index },

        Tile::Chance => MoveResult::LandedOnChance { tile_index },

        Tile::Property(p) => match p.owner {
            None => MoveResult::LandedOnProperty { tile_index },
            Some(owner) => {
                if owner == state.players[state.current_player_index].id {
                    MoveResult::NoEffect
                } else {
                    MoveResult::LandedOnOwnedProperty { tile_index, owner }
                }
            }
        },

        Tile::Railroad(r) => match r.owner {
            None => MoveResult::LandedOnProperty { tile_index },
            Some(owner) => {
                if owner == state.players[state.current_player_index].id {
                    MoveResult::NoEffect
                } else {
                    MoveResult::LandedOnOwnedProperty { tile_index, owner }
                }
            }
        },

        Tile::Utility(u) => match u.owner {
            None => MoveResult::LandedOnProperty { tile_index },
            Some(owner) => {
                if owner == state.players[state.current_player_index].id {
                    MoveResult::NoEffect
                } else {
                    MoveResult::LandedOnOwnedProperty { tile_index, owner }
                }
            }
        },

        Tile::Jail | Tile::FreeParking => {
            if passed_go {
                MoveResult::PassedGo
            } else {
                MoveResult::NoEffect
            }
        }
    }
}

/// Send the current player to jail — position 10, in_jail flag set.
pub fn send_to_jail(state: &mut GameState) {
    let player = &mut state.players[state.current_player_index];
    player.position = 10; // Jail is always tile 10
    player.in_jail = true;
    player.jail_turns = 0;
    state.turn_phase = TurnPhase::JailDecision;
}
