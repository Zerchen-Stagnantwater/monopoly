use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::state::{GameState, TurnPhase};
use monopoly_core::board::{Tile, ColorGroup};
use crate::screens::Screen;

// Board geometry
const BOARD_X: f32 = 20.0;
const BOARD_Y: f32 = 20.0;
const BOARD_SIZE: f32 = 700.0;
const CORNER_SIZE: f32 = 80.0;
const TILE_W: f32 = (BOARD_SIZE - 2.0 * CORNER_SIZE) / 9.0;
const TILE_H: f32 = CORNER_SIZE;

pub struct GameScreen {
    pub state: GameState,
    pub my_id: u8,
    pub tx: mpsc::Sender<ClientMessage>,
    bid_input: String,
}

impl GameScreen {
    pub fn new(state: GameState, my_id: u8, tx: mpsc::Sender<ClientMessage>) -> Self {
        Self { state, my_id, tx, bid_input: String::new() }
    }

pub fn update(&mut self) -> Option<Screen> {
    // Auction input — open to ALL players
    if matches!(self.state.turn_phase, TurnPhase::Auction { .. }) {
        if let Some(c) = get_char_pressed() {
            if c.is_ascii_digit() {
                self.bid_input.push(c);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.bid_input.pop();
        }
        if is_key_pressed(KeyCode::Enter) {
            if let Ok(amount) = self.bid_input.trim().parse::<u32>() {
                let _ = self.tx.send(ClientMessage::PlaceBid { amount });
                self.bid_input.clear();
            }
        }
        if is_key_pressed(KeyCode::P) {
            let _ = self.tx.send(ClientMessage::PassBid);
            self.bid_input.clear();
        }
        return None;
    }

    // All other actions — current player only
    if self.state.players[self.state.current_player_index].id == self.my_id {
        match self.state.turn_phase {
            TurnPhase::WaitingForRoll => {
                if is_key_pressed(KeyCode::Space) {
                    let _ = self.tx.send(ClientMessage::RollDice);
                }
            }
            TurnPhase::BuyDecision { .. } => {
                if is_key_pressed(KeyCode::B) {
                    let _ = self.tx.send(ClientMessage::BuyProperty);
                }
                if is_key_pressed(KeyCode::D) {
                    let _ = self.tx.send(ClientMessage::DeclineProperty);
                }
            }
            TurnPhase::PostRoll => {
                if is_key_pressed(KeyCode::E) {
                    let _ = self.tx.send(ClientMessage::EndTurn);
                }
            }
            TurnPhase::JailDecision => {
                if is_key_pressed(KeyCode::P) {
                    let _ = self.tx.send(ClientMessage::PayJailFine);
                }
                if is_key_pressed(KeyCode::R) {
                    let _ = self.tx.send(ClientMessage::RollForJail);
                }
            }
            TurnPhase::PayingRent { amount: _, to_player: _ } => {
                if is_key_pressed(KeyCode::E) {
                    let _ = self.tx.send(ClientMessage::PayRent);
                }
            }
            _ => {}
        }
    }

    None
}
    pub fn draw(&self) {
        draw_board(&self.state);
        draw_players(&self.state);
        draw_player_info(&self.state, self.my_id);
        draw_action_panel(&self.state, self.my_id);
        draw_bid_input(&self.state, &self.bid_input);
    }
}

fn draw_bid_input(state: &GameState, bid_input: &str) {
    if matches!(state.turn_phase, TurnPhase::Auction { .. }) {
        let px = BOARD_X + BOARD_SIZE + 20.0;
        let py = BOARD_Y + BOARD_SIZE - 100.0;
        draw_text("Your bid:", px, py, 20.0, BLACK);
        draw_rectangle_lines(px, py + 8.0, 200.0, 34.0, 2.0, BLUE);
        draw_text(
            &format!("${}", bid_input),
            px + 8.0, py + 30.0, 22.0, BLACK,
        );
    }
}

fn color_group_color(group: &ColorGroup) -> Color {
    match group {
        ColorGroup::Brown    => Color::from_rgba(139, 69, 19, 255),
        ColorGroup::LightBlue=> Color::from_rgba(173, 216, 230, 255),
        ColorGroup::Pink     => Color::from_rgba(255, 105, 180, 255),
        ColorGroup::Orange   => Color::from_rgba(255, 165, 0, 255),
        ColorGroup::Red      => Color::from_rgba(220, 20, 60, 255),
        ColorGroup::Yellow   => Color::from_rgba(255, 255, 0, 255),
        ColorGroup::Green    => Color::from_rgba(0, 128, 0, 255),
        ColorGroup::DarkBlue => Color::from_rgba(0, 0, 139, 255),
    }
}

/// Returns the (x, y, width, height, rotation_degrees) for each tile 0–39
fn tile_rect(index: usize) -> (f32, f32, f32, f32) {
    let b = BOARD_X;
    let t = BOARD_Y;
    let s = BOARD_SIZE;
    let c = CORNER_SIZE;
    let tw = TILE_W;
    let th = TILE_H;

    match index {
        // Bottom row right to left (0=Go at bottom-right corner)
        0  => (b + s - c, t + s - c, c, c),           // Go (corner)
        1  => (b + s - c - tw, t + s - th, tw, th),
        2  => (b + s - c - 2.0*tw, t + s - th, tw, th),
        3  => (b + s - c - 3.0*tw, t + s - th, tw, th),
        4  => (b + s - c - 4.0*tw, t + s - th, tw, th),
        5  => (b + s - c - 5.0*tw, t + s - th, tw, th),
        6  => (b + s - c - 6.0*tw, t + s - th, tw, th),
        7  => (b + s - c - 7.0*tw, t + s - th, tw, th),
        8  => (b + s - c - 8.0*tw, t + s - th, tw, th),
        9  => (b + s - c - 9.0*tw, t + s - th, tw, th),
        10 => (b, t + s - c, c, c),                    // Jail (corner)

        // Left column bottom to top
        11 => (b, t + s - c - th, th, tw),
        12 => (b, t + s - c - 2.0*tw, th, tw),
        13 => (b, t + s - c - 3.0*tw, th, tw),
        14 => (b, t + s - c - 4.0*tw, th, tw),
        15 => (b, t + s - c - 5.0*tw, th, tw),
        16 => (b, t + s - c - 6.0*tw, th, tw),
        17 => (b, t + s - c - 7.0*tw, th, tw),
        18 => (b, t + s - c - 8.0*tw, th, tw),
        19 => (b, t + s - c - 9.0*tw, th, tw),
        20 => (b, t, c, c),                            // Free Parking (corner)

        // Top row left to right
        21 => (b + c, t, tw, th),
        22 => (b + c + tw, t, tw, th),
        23 => (b + c + 2.0*tw, t, tw, th),
        24 => (b + c + 3.0*tw, t, tw, th),
        25 => (b + c + 4.0*tw, t, tw, th),
        26 => (b + c + 5.0*tw, t, tw, th),
        27 => (b + c + 6.0*tw, t, tw, th),
        28 => (b + c + 7.0*tw, t, tw, th),
        29 => (b + c + 8.0*tw, t, tw, th),
        30 => (b + s - c, t, c, c),                   // Go To Jail (corner)

        // Right column top to bottom
        31 => (b + s - th, t + c, th, tw),
        32 => (b + s - th, t + c + tw, th, tw),
        33 => (b + s - th, t + c + 2.0*tw, th, tw),
        34 => (b + s - th, t + c + 3.0*tw, th, tw),
        35 => (b + s - th, t + c + 4.0*tw, th, tw),
        36 => (b + s - th, t + c + 5.0*tw, th, tw),
        37 => (b + s - th, t + c + 6.0*tw, th, tw),
        38 => (b + s - th, t + c + 7.0*tw, th, tw),
        39 => (b + s - th, t + c + 8.0*tw, th, tw),
        _  => (0.0, 0.0, 0.0, 0.0),
    }
}

fn draw_board(state: &GameState) {
    // Board background
    draw_rectangle(BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE, Color::from_rgba(200, 230, 200, 255));

    // Draw each tile
    for (i, tile) in state.board.tiles.iter().enumerate() {
        let (x, y, w, h) = tile_rect(i);

        // Tile background
        draw_rectangle(x, y, w, h, WHITE);
        draw_rectangle_lines(x, y, w, h, 1.0, DARKGRAY);

        // Color group strip for properties
        if let Tile::Property(p) = tile {
            let c = color_group_color(&p.color_group);
            // Strip on the inner edge of the tile
            let strip = 12.0;
            match i {
                1..=9   => draw_rectangle(x, y, w, strip, c),          // bottom row — strip on top
                11..=19 => draw_rectangle(x + w - strip, y, strip, h, c), // left col — strip on right
                21..=29 => draw_rectangle(x, y + h - strip, w, strip, c), // top row — strip on bottom
                31..=39 => draw_rectangle(x, y, strip, h, c),          // right col — strip on left
                _ => {}
            }

            // House dots
            for house in 0..p.houses.min(4) {
                draw_circle(x + w/2.0 + (house as f32 - 1.5) * 8.0, y + h/2.0, 4.0, GREEN);
            }
            if p.houses == 5 {
                draw_rectangle(x + w/2.0 - 6.0, y + h/2.0 - 6.0, 12.0, 12.0, RED);
            }
        }

        // Tile name (abbreviated)
        let label = tile_label(tile);
        let font_size = 10.0;
        draw_text(&label, x + 2.0, y + h/2.0, font_size, BLACK);
    }

    // Center logo
    let cx = BOARD_X + BOARD_SIZE / 2.0;
    let cy = BOARD_Y + BOARD_SIZE / 2.0;
    draw_text("MONOPOLY", cx - 60.0, cy, 28.0, BLACK);
}

fn tile_label(tile: &Tile) -> String {
    match tile {
        Tile::Property(p)    => p.name.chars().take(8).collect(),
        Tile::Railroad(r)    => r.name.chars().take(8).collect(),
        Tile::Utility(u)     => u.name.chars().take(8).collect(),
        Tile::Tax(t)         => t.name.chars().take(8).collect(),
        Tile::Go             => "GO".to_string(),
        Tile::Jail           => "JAIL".to_string(),
        Tile::FreeParking    => "FREE".to_string(),
        Tile::GoToJail       => "GO JAIL".to_string(),
        Tile::CommunityChest => "COM CHST".to_string(),
        Tile::Chance         => "CHANCE".to_string(),
    }
}

fn draw_players(state: &GameState) {
    let token_colors = [RED, BLUE, GREEN, YELLOW, PURPLE, ORANGE];
    for (pi, player) in state.players.iter().enumerate() {
        if player.bankrupt { continue; }
        let (x, y, w, h) = tile_rect(player.position);
        let offset = (pi as f32) * 10.0;
        draw_circle(
            x + w / 2.0 + offset - 15.0,
            y + h / 2.0,
            8.0,
            token_colors[pi % token_colors.len()],
        );
    }
}

fn draw_player_info(state: &GameState, my_id: u8) {
    let px = BOARD_X + BOARD_SIZE + 20.0;
    let py = BOARD_Y;

    draw_text("PLAYERS", px, py + 30.0, 28.0, BLACK);

    for (i, player) in state.players.iter().enumerate() {
        let y = py + 70.0 + i as f32 * 100.0;
        let is_me = player.id == my_id;
        let is_current = i == state.current_player_index;

        let label_color = if is_current { BLUE } else { BLACK };
        let prefix = if is_me { "▶ " } else { "  " };

        draw_text(
            &format!("{}{}", prefix, player.name),
            px, y, 22.0, label_color,
        );
        draw_text(
            &format!("  ${}", player.money),
            px, y + 24.0, 20.0, DARKGREEN,
        );
        if player.in_jail {
            draw_text("  [IN JAIL]", px, y + 46.0, 18.0, RED);
        }
        if player.bankrupt {
            draw_text("  [BANKRUPT]", px, y + 46.0, 18.0, GRAY);
        }
    }
}
pub fn draw_action_panel(state: &GameState, my_id: u8) {
    let px = BOARD_X + BOARD_SIZE + 20.0;
    let py = BOARD_Y + BOARD_SIZE - 280.0;

    draw_line(px, py, px + 380.0, py, 1.0, DARKGRAY);
    draw_text("ACTIONS", px, py + 30.0, 24.0, BLACK);

    let current = &state.players[state.current_player_index];

    // Auction is open to all players regardless of whose turn it is
    if let TurnPhase::Auction { highest_bid, highest_bidder, .. } = &state.turn_phase {
        draw_text(&format!("AUCTION - Top bid: ${}", highest_bid), px, py + 60.0, 20.0, BLACK);
        draw_text(&format!("Leader: Player {}", highest_bidder), px, py + 85.0, 20.0, DARKGRAY);
        draw_text("Type amount + Enter to bid", px, py + 115.0, 18.0, BLACK);
        draw_text("[P] Pass on auction", px, py + 138.0, 18.0, RED);
        return;
    }

    // All other phases — only current player sees controls
    if current.id != my_id {
        draw_text(
            &format!("Waiting for {}...", current.name),
            px, py + 60.0, 20.0, DARKGRAY,
        );
        return;
    }

    match &state.turn_phase {
        TurnPhase::WaitingForRoll => {
            draw_text("[SPACE] Roll dice", px, py + 60.0, 20.0, BLACK);
        }
        TurnPhase::BuyDecision { tile_index } => {
            let price = match &state.board.tiles[*tile_index] {
                Tile::Property(p) => p.price,
                Tile::Railroad(r) => r.price,
                Tile::Utility(u)  => u.price,
                _ => 0,
            };
            draw_text(&format!("[B] Buy  (${})", price), px, py + 60.0, 20.0, DARKGREEN);
            draw_text("[D] Decline", px, py + 85.0, 20.0, RED);
        }
        TurnPhase::PostRoll => {
            draw_text("[E] End turn", px, py + 60.0, 20.0, BLACK);
        }
        TurnPhase::JailDecision => {
            draw_text("[P] Pay fine ($50)", px, py + 60.0, 20.0, BLACK);
            draw_text("[R] Roll for doubles", px, py + 85.0, 20.0, BLACK);
        }
        TurnPhase::PayingRent { amount, .. } => {
            draw_text(&format!("Pay rent: ${}", amount), px, py + 60.0, 20.0, RED);
            draw_text("[E] Confirm", px, py + 85.0, 20.0, BLACK);
        }
        TurnPhase::EndTurn => {
            draw_text("Ending turn...", px, py + 60.0, 20.0, DARKGRAY);
        }
        TurnPhase::Auction { .. } => {} // handled above
    }
}
