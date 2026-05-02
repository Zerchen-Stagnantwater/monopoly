use serde::{Deserialize, Serialize};
use crate::{Board, Player};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnPhase {
    WaitingForRoll,
    BuyDecision { tile_index: usize },
    Auction { tile_index: usize, highest_bid: u32, highest_bidder: u8 },
    PayingRent { amount: u32, to_player: u8 },
    JailDecision,
    PostRoll,
    EndTurn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Board,
    pub players: Vec<Player>,
    pub current_player_index: usize,
    pub turn_phase: TurnPhase,
    pub turn_number: u32,
    pub last_roll: Option<(u8, u8)>,
    pub free_parking_pot: u32,
    pub game_over: bool,
    pub winner: Option<u8>,
    // Building supply lives here so it's under one mutable borrow
    pub houses_remaining: u8,
    pub hotels_remaining: u8,
    pub auction_passers: Vec<u8>,
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
        self.auction_passers.clear();
    }
}
