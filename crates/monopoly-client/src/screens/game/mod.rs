mod board;
mod sidebar;
mod tooltip;
mod card_panel;
mod card_detail;
mod trade;

use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::state::{GameState, TurnPhase};
use crate::screens::Screen;
use crate::theme::Theme;

use crate::screens::game::board::BOARD_X;
pub use trade::{TradeScreenState, TradeInput};

pub struct GameScreen {
    pub state: GameState,
    pub my_id: u8,
    pub tx: mpsc::Sender<ClientMessage>,
    pub theme: Theme,
    bid_input: String,
    event_log: Vec<String>,
    card_panel_open: bool,
    card_panel_scroll: f32,
    selected_card: Option<usize>,
    trade_screen: Option<TradeScreenState>,
}

impl GameScreen {
    pub fn new(state: GameState, my_id: u8, tx: mpsc::Sender<ClientMessage>, theme: Theme) -> Self {
        Self {
            state,
            my_id,
            tx,
            theme,
            bid_input: String::new(),
            event_log: Vec::new(),
            card_panel_open: false,
            card_panel_scroll: 0.0,
            selected_card: None,
            trade_screen: None,
        }
    }

    pub fn push_event(&mut self, msg: String) {
        self.event_log.push(msg);
        if self.event_log.len() > 6 {
            self.event_log.remove(0);
        }
    }

    pub fn update(&mut self) -> Option<Screen> {
        // Toggle card panel
        if is_key_pressed(KeyCode::C) {
            self.card_panel_open = !self.card_panel_open;
            self.card_panel_scroll = 0.0;
        }

        // Scroll card panel
        if self.card_panel_open {
            let (_, wheel_y) = mouse_wheel();
            if wheel_y != 0.0 {
                self.card_panel_scroll -= wheel_y * 30.0;
                if self.card_panel_scroll < 0.0 {
                    self.card_panel_scroll = 0.0;
                }
            }
        }

        // Auction — open to all players
        if matches!(self.state.turn_phase, TurnPhase::Auction { .. }) {
            if let Some(c) = get_char_pressed() {
                if c.is_ascii_digit() { self.bid_input.push(c); }
            }
            if is_key_pressed(KeyCode::Backspace) { self.bid_input.pop(); }
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

        // Trade screen
        if let Some(ref mut trade) = self.trade_screen {
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                let active: Vec<u8> = self.state.players.iter()
                    .filter(|p| p.id != self.my_id && !p.bankrupt)
                    .map(|p| p.id)
                    .collect();
                if let Some(pos) = active.iter().position(|&id| id == trade.target_player) {
                    let next = if is_key_pressed(KeyCode::Right) {
                        (pos + 1) % active.len()
                    } else {
                        (pos + active.len() - 1) % active.len()
                    };
                    trade.target_player = active[next];
                    trade.requested_properties.clear();
                }
            }

            if is_key_pressed(KeyCode::Tab) {
                trade.active_input = match trade.active_input {
                    TradeInput::OfferedMoney   => TradeInput::RequestedMoney,
                    TradeInput::RequestedMoney => TradeInput::OfferedMoney,
                };
            }
            
            let (_, wheel_y) = mouse_wheel();
            if wheel_y != 0.0 {
                let (mx, _) = mouse_position();
                let panel_w = 860.0;
                let px = screen_width() / 2.0 - panel_w / 2.0;
                let col_w = (panel_w - 40.0) / 2.0;
                let mid_x = px + 12.0 + col_w + 16.0;
                if mx < mid_x {
                    trade.offer_scroll = (trade.offer_scroll - wheel_y * 30.0).max(0.0);
                } else {
                    trade.request_scroll = (trade.request_scroll - wheel_y * 30.0).max(0.0);
                }
            }

            if let Some(c) = get_char_pressed() {
                if c.is_ascii_digit() {
                    match trade.active_input {
                        TradeInput::OfferedMoney => {
                            if trade.offered_money_input == "0" {
                                trade.offered_money_input = c.to_string();
                            } else {
                                trade.offered_money_input.push(c);
                            }
                        }
                        TradeInput::RequestedMoney => {
                            if trade.requested_money_input == "0" {
                                trade.requested_money_input = c.to_string();
                            } else {
                                trade.requested_money_input.push(c);
                            }
                        }
                    }
                }
            }

            if is_key_pressed(KeyCode::Backspace) {
                match trade.active_input {
                    TradeInput::OfferedMoney => {
                        trade.offered_money_input.pop();
                        if trade.offered_money_input.is_empty() {
                            trade.offered_money_input = "0".to_string();
                        }
                    }
                    TradeInput::RequestedMoney => {
                        trade.requested_money_input.pop();
                        if trade.requested_money_input.is_empty() {
                            trade.requested_money_input = "0".to_string();
                        }
                    }
                }
            }

            if is_key_pressed(KeyCode::Enter) {
                let offered_money = trade.offered_money_input.parse::<u32>().unwrap_or(0);
                let requested_money = trade.requested_money_input.parse::<u32>().unwrap_or(0);
                let to_player = trade.target_player;
                let offered_properties = trade.offered_properties.clone();
                let requested_properties = trade.requested_properties.clone();
                let _ = trade;
                let _ = self.tx.send(ClientMessage::ProposeTrade {
                    to_player,
                    offered_properties,
                    offered_money,
                    requested_properties,
                    requested_money,
                });
                self.trade_screen = None;
                return None;
            }

            if is_key_pressed(KeyCode::Escape) {
                let _ = trade;
                self.trade_screen = None;
                return None;
            }

            // Property click selection
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                let panel_w = 860.0;
                let panel_h = 520.0;
                let px = screen_width() / 2.0 - panel_w / 2.0;
                let py = screen_height() / 2.0 - panel_h / 2.0;
                let col_w = (panel_w - 40.0) / 2.0;
                let left_x  = px + 12.0;
                let right_x = px + 12.0 + col_w + 16.0;
                let col_y   = py + 76.0;
                let card_w  = 80.0;
                let card_h  = 80.0;
                let card_gap = 6.0;
                let prop_y  = col_y + 60.0;
                let prop_area_h = panel_h - 140.0;
                let cols = ((col_w - 16.0 + card_gap) / (card_w + card_gap)).floor() as usize;

                let my_props: Vec<usize> = self.state.players.iter()
                    .find(|p| p.id == self.my_id)
                    .map(|p| p.properties.clone())
                    .unwrap_or_default();
 
                for (i, &tile_index) in my_props.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let card_x = left_x + col as f32 * (card_w + card_gap);
                    let card_y = prop_y + row as f32 * (card_h + card_gap) - trade.offer_scroll;
                    if card_y + card_h < prop_y || card_y > prop_y + prop_area_h { continue; }
                    if mx >= card_x && mx <= card_x + card_w &&
                    my >= card_y && my <= card_y + card_h {
                        if trade.offered_properties.contains(&tile_index) {
                            trade.offered_properties.retain(|&t| t != tile_index);
                        } else {
                            trade.offered_properties.push(tile_index);
                        }
                    }
                }

                let target_props: Vec<usize> = self.state.players.iter()
                    .find(|p| p.id == trade.target_player)
                    .map(|p| p.properties.clone())
                    .unwrap_or_default();
                
                for (i, &tile_index) in target_props.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let card_x = right_x + col as f32 * (card_w + card_gap);
                    let card_y = prop_y + row as f32 * (card_h + card_gap) - trade.request_scroll;
                    if card_y + card_h < prop_y || card_y > prop_y + prop_area_h { continue; }
                    if mx >= card_x && mx <= card_x + card_w &&
                    my >= card_y && my <= card_y + card_h {
                        if trade.requested_properties.contains(&tile_index) {
                            trade.requested_properties.retain(|&t| t != tile_index);
                        } else {
                            trade.requested_properties.push(tile_index);
                        }
                    }
                }

            }

            return None;
        }

        // Open trade screen
        if matches!(self.state.turn_phase, TurnPhase::PostRoll) &&
            self.state.players[self.state.current_player_index].id == self.my_id &&
            is_key_pressed(KeyCode::T) {
            let target = self.state.players.iter()
                .find(|p| p.id != self.my_id && !p.bankrupt)
                .map(|p| p.id);
            if let Some(target_id) = target {
                self.trade_screen = Some(TradeScreenState::new(target_id));
                self.selected_card = None;
                self.card_panel_open = false;
            }
        }

        // Card detail actions
        if let Some(tile_index) = self.selected_card {
            if is_key_pressed(KeyCode::M) {
                let _ = self.tx.send(ClientMessage::Mortgage { tile_index });
                self.selected_card = None;
            }
            if is_key_pressed(KeyCode::U) {
                let _ = self.tx.send(ClientMessage::Unmortgage { tile_index });
                self.selected_card = None;
            }
            if is_key_pressed(KeyCode::B) {
                let _ = self.tx.send(ClientMessage::BuildHouse { tile_index });
            }
        }

        // Escape closes detail view
        if is_key_pressed(KeyCode::Escape) {
            self.selected_card = None;
        }

        // Click on card panel
        if self.card_panel_open && is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if let Some(tile_index) = card_panel::clicked_card(
                &self.state, self.my_id, mx, my, self.card_panel_scroll,
            ) {
                self.selected_card = if self.selected_card == Some(tile_index) {
                    None
                } else {
                    Some(tile_index)
                };
            }
        }

        // Accept/reject trade
        if is_key_pressed(KeyCode::Y) {
            let _ = self.tx.send(ClientMessage::AcceptTrade);
        }
        if is_key_pressed(KeyCode::N) {
            let _ = self.tx.send(ClientMessage::RejectTrade);
        }

        // Current player actions
        if self.state.players[self.state.current_player_index].id == self.my_id {
            match self.state.turn_phase {
                TurnPhase::WaitingForRoll => {
                    if is_key_pressed(KeyCode::Space) {
                        let _ = self.tx.send(ClientMessage::RollDice);
                    }
                }
                TurnPhase::BuyDecision { .. } => {
                    if is_key_pressed(KeyCode::B) && self.selected_card.is_none() {
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
        let t = &self.theme;
        let (mx, my) = mouse_position();

        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), t.window_bg);
        board::draw_board(&self.state, t);
        board::draw_players(&self.state, t);
        sidebar::draw_sidebar(&self.state, self.my_id, t, &self.bid_input, &self.event_log);

        if self.card_panel_open {
            card_panel::draw_card_panel(&self.state, self.my_id, t, self.card_panel_scroll);
        }

        if let Some(tile_index) = self.selected_card {
            card_detail::draw_card_detail(&self.state, tile_index, self.my_id, t);
        }

        if let Some(ref trade) = self.trade_screen {
            trade::draw_trade_screen(&self.state, self.my_id, trade, t);
        }
        if self.trade_screen.is_none() && self.selected_card.is_none() {
            if let Some(tile_index) = tooltip::hovered_tile(mx, my) {
                tooltip::draw_tile_tooltip(&self.state, tile_index, mx, my, t);
            }
        }

        let hint = if self.card_panel_open { "[C] Close cards" } else { "[C] My cards" };
        draw_text(hint, BOARD_X, screen_height() - 8.0, t.small_size, t.panel_subtext);
    }
}
