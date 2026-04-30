use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u8,
    pub name: String,
    pub token: Token,
    pub money: u32,
    pub position: usize,
    pub in_jail: bool,
    pub jail_turns: u8,        // how many turns spent in jail
    pub get_out_of_jail: u8,   // number of get out of jail free cards held
    pub bankrupt: bool,
    pub properties: Vec<usize>, // tile indices of owned properties
}

impl Player {
    pub fn new(id: u8, name: String, token: Token, starting_money: u32) -> Self {
        Self {
            id,
            name,
            token,
            money: starting_money,
            position: 0,
            in_jail: false,
            jail_turns: 0,
            get_out_of_jail: 0,
            bankrupt: false,
            properties: Vec::new(),
        }
    }

    pub fn is_solvent(&self) -> bool {
        !self.bankrupt
    }
}

/// The eight classic Monopoly tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Token {
    Battleship,
    Boot,
    Car,
    Dog,
    Hat,
    Iron,
    Thimble,
    Wheelbarrow,
}
