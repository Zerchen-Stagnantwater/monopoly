use serde::{Deserialize, Serialize};

/// All the numerical and behavioral knobs that define a game mode.
/// Standard Monopoly is one implementation — your custom modes override what they need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,

    // --- Money ---
    pub starting_money: u32,
    pub go_salary: u32,         // collected when passing or landing on Go
    pub jail_fine: u32,         // cost to get out of jail by paying

    // --- Jail ---
    pub max_jail_turns: u8,     // forced out after this many turns in jail
    pub doubles_to_exit_jail: bool, // can roll doubles to exit jail for free

    // --- Buildings ---
    pub max_houses: u8,         // total house supply (standard: 32)
    pub max_hotels: u8,         // total hotel supply (standard: 12)
    pub houses_per_hotel: u8,   // houses returned when building a hotel (standard: 4)

    // --- Auctions ---
    pub auction_enabled: bool,  // if false, unowned property is simply not bought

    // --- Free Parking ---
    pub free_parking_pot: bool, // house rule: taxes go to center, collected on free parking

    // --- Bankruptcy ---
    pub debt_to_bank_eliminates: bool, // bankrupt to bank = eliminated (always true in standard)
}

impl RuleSet {
    /// The official Hasbro standard ruleset.
    pub fn standard() -> Self {
        Self {
            name: String::from("Standard Monopoly"),
            starting_money: 1500,
            go_salary: 200,
            jail_fine: 50,
            max_jail_turns: 3,
            doubles_to_exit_jail: true,
            max_houses: 32,
            max_hotels: 12,
            houses_per_hotel: 4,
            auction_enabled: true,
            free_parking_pot: false,
            debt_to_bank_eliminates: true,
        }
    }
}
