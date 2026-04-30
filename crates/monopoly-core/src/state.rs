use serde::{Deserialize, Serialize};
use crate::{Board, Player};

/// Which phase of a turn we're currently in.
/// This is the state machine that drives the whole game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnPhase {
    /// Player must roll the dice
    WaitingForRoll,
    /// Player landed on an unowned property — buy or auction
    BuyDecision { tile_index: usize },
    /// An auction is in progress
    Auction { tile_index: usize, highest_bid: u32, highest_bidder: u8 },
    /// Player must pay rent
    PayingRent { amount: u32, to_player: u8 },
    /// Player is in jail — choose to pay, use card, or roll doubles
    JailDecision,
    /// Player may build houses/hotels, mortgage, or trade before ending turn
    PostRoll,
    /// Turn is over, moving to next player
    EndTurn,
}

/// The full authoritative game state — lives on the server.
/// Clients receive a snapshot of this after every state change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub players: Vec<Player>,
    pub current_player_index: usize,
    pub turn_phase: TurnPhase,
    pub turn_number: u32,
    pub last_roll: Option<(u8, u8)>,   // the two dice values
    pub free_parking_pot: u32,         // optional house rule, 0 in standard
    pub game_over: bool,
    pub winner: Option<u8>,            // player id of winner
}

impl GameState {
    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player_index]
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

    pub fn active_players(&self) -> Vec<&Player> {
        self.players.iter().filter(|p| !p.bankrupt).collect()
    }

    pub fn advance_turn(&mut self) {
        let total = self.players.len();
        self.current_player_index = (self.current_player_index + 1) % total;
        self.turn_phase = TurnPhase::WaitingForRoll;
        self.turn_number += 1;
        self.last_roll = None;
    }
}
