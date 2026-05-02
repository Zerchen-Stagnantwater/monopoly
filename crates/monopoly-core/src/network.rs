use serde::{Deserialize, Serialize};
use crate::player::Token;
use crate::state::GameState;

/// Messages sent from CLIENT → SERVER
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    // --- Lobby ---
    Connect { addr: String },
    /// First message sent after connecting
    Join { name: String, token: Token },
    /// Host triggers game start
    StartGame,

    // --- Turn actions ---
    /// Roll the dice (only valid in WaitingForRoll phase)
    RollDice,
    /// Buy the property the player landed on
    BuyProperty,
    /// Decline to buy — triggers auction if enabled
    DeclineProperty,
    /// Submit an auction bid
    PlaceBid { amount: u32 },
    /// Pass on bidding in an auction
    PassBid,
    /// End the auction (host only, or auto after all pass)
    FinalizeAuction,
    PayRent,
    // --- Jail ---
    /// Pay the jail fine
    PayJailFine,
    /// Use a get out of jail free card
    UseJailCard,
    /// Roll dice trying for doubles to escape jail
    RollForJail,

    // --- Post roll actions ---
    /// Build a house or hotel on a property
    BuildHouse { tile_index: usize },
    /// Sell a house or hotel back to the bank
    SellHouse { tile_index: usize },
    /// Mortgage a property
    Mortgage { tile_index: usize },
    /// Unmortgage a property
    Unmortgage { tile_index: usize },
    /// Propose a trade to another player
    ProposeTrade {
        to_player: u8,
        offered_properties: Vec<usize>,
        offered_money: u32,
        requested_properties: Vec<usize>,
        requested_money: u32,
    },
    /// Accept a trade that was proposed to you
    AcceptTrade,
    /// Reject a trade
    RejectTrade,
    /// End your turn
    EndTurn,

    // --- Any time ---
    /// Declare bankruptcy
    DeclareBankruptcy,
}

/// Messages sent from SERVER → CLIENT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // --- Lobby ---
    /// Sent to a client immediately after they successfully join
    JoinAck { assigned_id: u8 },
    /// Sent to a newly joined player with the full current lobby roster
    LobbyState { players: Vec<(u8, String, Token)> }, 
    /// Broadcast to all when a new player joins
    PlayerJoined { id: u8, name: String, token: Token },
    /// Broadcast to all when a player disconnects in lobby
    PlayerLeft { id: u8 },
    /// Sent when the host rejects a join (lobby full, game started, etc.)
    JoinRejected { reason: String },

    // --- Game state ---
    /// Full state snapshot — sent to all clients after every state change
    StateUpdate { state: GameState },
    /// Sent to all when game starts
    GameStarted,
    /// Sent to all when game ends
    GameOver { winner_id: u8, winner_name: String },

    // --- Action results ---
    /// An action was rejected — tells the client why
    ActionRejected { reason: String },

    // --- Trade ---
    /// Sent to the target player when a trade is proposed
    TradeProposed {
        from_player: u8,
        offered_properties: Vec<usize>,
        offered_money: u32,
        requested_properties: Vec<usize>,
        requested_money: u32,
    },
    /// Broadcast when a trade is accepted
    TradeAccepted,
    /// Broadcast when a trade is rejected
    TradeRejected,

    // --- Chat / events ---
    /// Game event log message — "Player X bought Boardwalk" etc.
    EventLog { message: String },
}

/// A framed packet — length prefix + bincode payload.
/// We use this to delimit messages over a raw TCP stream.
pub struct Packet;

impl Packet {
    /// Encode a message to bytes: 4-byte little-endian length + bincode payload.
    pub fn encode<T: Serialize>(msg: &T) -> anyhow::Result<Vec<u8>> {
        let payload = bincode::serialize(msg)?;
        let len = payload.len() as u32;
        let mut buf = Vec::with_capacity(4 + payload.len());
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    /// Decode a message from a length-prefixed byte slice.
    pub fn decode<T: for<'de> Deserialize<'de>>(buf: &[u8]) -> anyhow::Result<T> {
        Ok(bincode::deserialize(buf)?)
    }
}
