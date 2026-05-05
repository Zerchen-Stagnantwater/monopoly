use std::sync::mpsc;
use macroquad::prelude::*;
use monopoly_core::network::ClientMessage;
use monopoly_core::state::{GameState, TurnPhase};
use monopoly_core::board::{Tile, ColorGroup};
use crate::screens::Screen;
use crate::theme::{Theme, draw_card};

// Board geometry
const BOARD_X: f32 = 20.0;
const BOARD_Y: f32 = 20.0;
const BOARD_SIZE: f32 = 680.0;
const CORNER_SIZE: f32 = 76.0;
const TILE_W: f32 = (BOARD_SIZE - 2.0 * CORNER_SIZE) / 9.0;
const TILE_H: f32 = CORNER_SIZE;

// Sidebar
const SIDE_X: f32 = BOARD_X + BOARD_SIZE + 16.0;
const SIDE_W: f32 = 340.0;

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
#[derive(Debug, Clone)]
pub struct TradeScreenState {
    pub target_player: u8,
    pub offered_properties: Vec<usize>,
    pub offered_money_input: String,
    pub requested_properties: Vec<usize>,
    pub requested_money_input: String,
    pub active_input: TradeInput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeInput {
    OfferedMoney,
    RequestedMoney,
}

impl TradeScreenState {
    pub fn new(target_player: u8) -> Self {
        Self {
            target_player,
            offered_properties: Vec::new(),
            offered_money_input: String::from("0"),
            requested_properties: Vec::new(),
            requested_money_input: String::from("0"),
            active_input: TradeInput::OfferedMoney,
        }
    }
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
            selected_card:None,
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
        // Auction — open to all players
        if is_key_pressed(KeyCode::C) {
            self.card_panel_open = !self.card_panel_open;
            self.card_panel_scroll = 0.0;
        }

        if self.card_panel_open{
            let(_, wheel_y) = mouse_wheel();
            if wheel_y != 0.0{
                self.card_panel_scroll -= wheel_y * 30.0;
                if self.card_panel_scroll < 0.0 {
                    self.card_panel_scroll = 0.0;
                }
            }
        }
        // Detect click on a mini card in the panel
        if self.card_panel_open && is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if let Some(tile_index) = clicked_card(
                &self.state,
                self.my_id,
                mx, my,
                self.card_panel_scroll,
                ) {
                    // Toggle — clicking same card closes detail view
                    if self.selected_card == Some(tile_index) {
                        self.selected_card = None;
                    } else {
                        self.selected_card = Some(tile_index);
                    }
                }
        }

        // Escape closes detail view
        if is_key_pressed(KeyCode::Escape) {
            self.selected_card = None;
        }   

        // Card detail actions — only when a card is selected
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
        // Open trade screen with T during PostRoll
        if matches!(self.state.turn_phase, TurnPhase::PostRoll) &&
            self.state.players[self.state.current_player_index].id == self.my_id &&
            is_key_pressed(KeyCode::T) &&
            self.trade_screen.is_none()
        {   
            // Default to first other active player
            let target = self.state.players.iter()
                .find(|p| p.id != self.my_id && !p.bankrupt)
                .map(|p| p.id);
            if let Some(target_id) = target {
                self.trade_screen = Some(TradeScreenState::new(target_id));
                self.selected_card = None;
                self.card_panel_open = false;
            }
        }

        // Trade screen input
        if let Some(ref mut trade) = self.trade_screen {
            // Switch target player with left/right
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

            // Switch active money input with Tab
            if is_key_pressed(KeyCode::Tab) {
                trade.active_input = match trade.active_input {
                    TradeInput::OfferedMoney   => TradeInput::RequestedMoney,
                    TradeInput::RequestedMoney => TradeInput::OfferedMoney,
                };
            }

            // Type money amount
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
            // Send offer with Enter
            if is_key_pressed(KeyCode::Enter) {
                let offered_money = trade.offered_money_input.parse::<u32>().unwrap_or(0);
                let requested_money = trade.requested_money_input.parse::<u32>().unwrap_or(0);
                let to_player = trade.target_player;
                let offered_properties = trade.offered_properties.clone();
                let requested_properties = trade.requested_properties.clone();
                let _ = trade;  // release borrow before mutating self
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

            // Cancel with Escape
            if is_key_pressed(KeyCode::Escape) {
                let _ = trade;
                self.trade_screen = None;
                return None;
            }
            // Click properties — all inside same block, using trade directly
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                let panel_w = 860.0;
                let panel_h = 520.0;
                let px = screen_width() / 2.0 - panel_w / 2.0;
                let py = screen_height() / 2.0 - panel_h / 2.0;
                let col_w = (panel_w - 40.0) / 2.0;
                let left_x = px + 12.0;
                let right_x = px + 12.0 + col_w + 16.0;
                let col_y = py + 76.0;

                let my_props: Vec<usize> = self.state.players.iter()
                    .find(|p| p.id == self.my_id)
                    .map(|p| p.properties.clone())
                    .unwrap_or_default();

                for (i, &tile_index) in my_props.iter().enumerate() {
                    let card_x = left_x + (i as f32 % 4.0) * 88.0;
                    let card_y = col_y + 60.0 + (i as f32 / 4.0).floor() * 90.0;
                    if mx >= card_x && mx <= card_x + 80.0 &&
                        my >= card_y && my <= card_y + 80.0 {
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
                    let card_x = right_x + (i as f32 % 4.0) * 88.0;
                    let card_y = col_y + 60.0 + (i as f32 / 4.0).floor() * 90.0;
                    if mx >= card_x && mx <= card_x + 80.0 &&
                        my >= card_y && my <= card_y + 80.0 {
                            if trade.requested_properties.contains(&tile_index) {
                                trade.requested_properties.retain(|&t| t != tile_index);
                            } else {
                                trade.requested_properties.push(tile_index);
                            }
                    }
                }   
            }   
        }
        // Handle incoming trade proposal — show accept/reject
        if matches!(self.state.turn_phase, TurnPhase::PostRoll) {
            if is_key_pressed(KeyCode::Y) {
                let _ = self.tx.send(ClientMessage::AcceptTrade);
            }
            if is_key_pressed(KeyCode::N) {
                let _ = self.tx.send(ClientMessage::RejectTrade);
            }
        } 
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

        // Current player actions
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
        let t = &self.theme;
        let (mx, my) = mouse_position();
        // Soft ambient background — no harsh white
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), t.window_bg);

        draw_board(&self.state, t);
        draw_players(&self.state, t);
        draw_sidebar(&self.state, self.my_id, t, &self.bid_input, &self.event_log);
        
        if self.card_panel_open {
            draw_card_panel(&self.state, self.my_id, t, self.card_panel_scroll);
        }
       
        if let Some(tile_index) = self.selected_card{
            draw_card_detail(&self.state, tile_index,self.my_id, t );
        }

        if let Some(ref trade) = self.trade_screen{
            draw_trade_screen(&self.state, self.my_id, trade, t);
        }

        if let Some(tile_index) = hovered_tile(mx, my){
            draw_tile_tooltip(&self.state, tile_index, mx, my, t);
        }
        
        let hint = if self.card_panel_open{ "[C] Close cards" } else { "[C] My cards" };
        draw_text(hint, BOARD_X, screen_height() - 8.0, t.small_size, t.panel_subtext );
    }
}

// --- Board ---

fn group_color(group: &ColorGroup, t: &Theme) -> Color {
    match group {
        ColorGroup::Brown    => t.group_brown,
        ColorGroup::LightBlue => t.group_light_blue,
        ColorGroup::Pink     => t.group_pink,
        ColorGroup::Orange   => t.group_orange,
        ColorGroup::Red      => t.group_red,
        ColorGroup::Yellow   => t.group_yellow,
        ColorGroup::Green    => t.group_green,
        ColorGroup::DarkBlue => t.group_dark_blue,
    }
}

fn tile_rect(index: usize) -> (f32, f32, f32, f32) {
    let b = BOARD_X;
    let t = BOARD_Y;
    let s = BOARD_SIZE;
    let c = CORNER_SIZE;
    let tw = TILE_W;
    let th = TILE_H;

    match index {
        0  => (b + s - c, t + s - c, c, c),
        1  => (b + s - c - tw,       t + s - th, tw, th),
        2  => (b + s - c - 2.0*tw,   t + s - th, tw, th),
        3  => (b + s - c - 3.0*tw,   t + s - th, tw, th),
        4  => (b + s - c - 4.0*tw,   t + s - th, tw, th),
        5  => (b + s - c - 5.0*tw,   t + s - th, tw, th),
        6  => (b + s - c - 6.0*tw,   t + s - th, tw, th),
        7  => (b + s - c - 7.0*tw,   t + s - th, tw, th),
        8  => (b + s - c - 8.0*tw,   t + s - th, tw, th),
        9  => (b + s - c - 9.0*tw,   t + s - th, tw, th),
        10 => (b, t + s - c, c, c),
        11 => (b, t + s - c - th,      th, tw),
        12 => (b, t + s - c - 2.0*tw,  th, tw),
        13 => (b, t + s - c - 3.0*tw,  th, tw),
        14 => (b, t + s - c - 4.0*tw,  th, tw),
        15 => (b, t + s - c - 5.0*tw,  th, tw),
        16 => (b, t + s - c - 6.0*tw,  th, tw),
        17 => (b, t + s - c - 7.0*tw,  th, tw),
        18 => (b, t + s - c - 8.0*tw,  th, tw),
        19 => (b, t + s - c - 9.0*tw,  th, tw),
        20 => (b, t, c, c),
        21 => (b + c,           t, tw, th),
        22 => (b + c + tw,      t, tw, th),
        23 => (b + c + 2.0*tw,  t, tw, th),
        24 => (b + c + 3.0*tw,  t, tw, th),
        25 => (b + c + 4.0*tw,  t, tw, th),
        26 => (b + c + 5.0*tw,  t, tw, th),
        27 => (b + c + 6.0*tw,  t, tw, th),
        28 => (b + c + 7.0*tw,  t, tw, th),
        29 => (b + c + 8.0*tw,  t, tw, th),
        30 => (b + s - c, t, c, c),
        31 => (b + s - th, t + c,           th, tw),
        32 => (b + s - th, t + c + tw,      th, tw),
        33 => (b + s - th, t + c + 2.0*tw,  th, tw),
        34 => (b + s - th, t + c + 3.0*tw,  th, tw),
        35 => (b + s - th, t + c + 4.0*tw,  th, tw),
        36 => (b + s - th, t + c + 5.0*tw,  th, tw),
        37 => (b + s - th, t + c + 6.0*tw,  th, tw),
        38 => (b + s - th, t + c + 7.0*tw,  th, tw),
        39 => (b + s - th, t + c + 8.0*tw,  th, tw),
        _  => (0.0, 0.0, 0.0, 0.0),
    }
}
/// Returns the tile index the mouse is currently hovering over, if any.
fn hovered_tile(mouse_x: f32, mouse_y: f32) -> Option<usize> {
    for i in 0..40 {
        let (x, y, w, h) = tile_rect(i);
        if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
            return Some(i);
        }
    }
    None
}

fn draw_board(state: &GameState, t: &Theme) {
    // Board background
    draw_rectangle(BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE, t.board_bg);
    draw_rectangle_lines(BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE, 2.0, t.board_border);

    for (i, tile) in state.board.tiles.iter().enumerate() {
        let (x, y, w, h) = tile_rect(i);

        // Tile base
        draw_rectangle(x, y, w, h, t.tile_bg);
        draw_rectangle_lines(x, y, w, h, t.tile_border_thickness, t.tile_border);

        // Color strip for properties
        if let Tile::Property(p) = tile {
            let c = group_color(&p.color_group, t);
            let strip = t.color_strip_height;
            match i {
                1..=9   => draw_rectangle(x, y, w, strip, c),
                11..=19 => draw_rectangle(x + w - strip, y, strip, h, c),
                21..=29 => draw_rectangle(x, y + h - strip, w, strip, c),
                31..=39 => draw_rectangle(x, y, strip, h, c),
                _ => {}
            }

            // Owner flag — small triangle in corner
            if let Some(owner_id) = p.owner {
                let oc = player_color(owner_id, t);
                draw_circle(x + w - 8.0, y + 8.0, 5.0, oc);
            }
            // Houses
            if p.houses > 0 && p.houses < 5 {
                for house in 0..p.houses {
                    draw_rectangle(
                        x + 2.0 + house as f32 * 8.0,
                        y + h - 11.0,
                        6.0, 6.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255),
                    );
                    draw_rectangle_lines(
                        x + 2.0 + house as f32 * 8.0,
                        y + h - 11.0,
                        6.0, 6.0,
                        1.0,
                        Color::from_rgba(0x00, 0x80, 0x00, 255),
                    );
                }
            } else if p.houses == 5 {
                draw_rectangle(x + 2.0, y + h - 13.0, w - 4.0, 10.0,
                    Color::from_rgba(0xe7, 0x4c, 0x3c, 255));
                draw_rectangle_lines(x + 2.0, y + h - 13.0, w - 4.0, 10.0, 1.0,
                    Color::from_rgba(0x8b, 0x00, 0x00, 255));
            }
        }

        // Owner dot for railroads and utilities
        match tile {
            Tile::Railroad(r) => {
                if let Some(owner_id) = r.owner {
                    let oc = player_color(owner_id, t);
                    draw_circle(x + w / 2.0, y + h / 2.0, 5.0, oc);
                }
            }
            Tile::Utility(u) => {
                if let Some(owner_id) = u.owner {
                    let oc = player_color(owner_id, t);
                    draw_circle(x + w / 2.0, y + h / 2.0, 5.0, oc);
                }
            }
            _ => {}
        }

        // Tile label
        let label = tile_label(tile);
        draw_text(&label, x + 2.0, y + h / 2.0 + 4.0, 9.0, t.tile_border);
    }

    // Center
    let cx = BOARD_X + BOARD_SIZE / 2.0;
    let cy = BOARD_Y + BOARD_SIZE / 2.0;
    draw_text("MONOPOLY", cx - 55.0, cy - 8.0, 26.0, t.board_border);
    draw_text(
        &format!("Turn {}", 0),
        cx - 30.0, cy + 18.0, 14.0,
        Color::new(t.board_border.r, t.board_border.g, t.board_border.b, 0.5),
    );
}

fn tile_label(tile: &Tile) -> String {
    match tile {
        Tile::Property(p)    => p.name.chars().take(7).collect(),
        Tile::Railroad(r)    => r.name.chars().take(7).collect(),
        Tile::Utility(u)     => u.name.chars().take(7).collect(),
        Tile::Tax(t)         => t.name.chars().take(7).collect(),
        Tile::Go             => "GO".into(),
        Tile::Jail           => "JAIL".into(),
        Tile::FreeParking    => "FREE".into(),
        Tile::GoToJail       => "GO JAIL".into(),
        Tile::CommunityChest => "COM CHT".into(),
        Tile::Chance         => "CHANCE".into(),
    }
}

fn draw_players(state: &GameState, t: &Theme) {
    for (pi, player) in state.players.iter().enumerate() {
        if player.bankrupt { continue; }
        let (x, y, w, h) = tile_rect(player.position);
        let color = player_color(player.id, t);
        let offset_x = (pi as f32 % 3.0) * 12.0 - 12.0;
        let offset_y = (pi as f32 / 3.0).floor() * 12.0 - 6.0;

        // Glowing token
        draw_circle(
            x + w / 2.0 + offset_x,
            y + h / 2.0 + offset_y,
            7.0,
            Color::new(color.r, color.g, color.b, 0.3),
        );
        draw_circle(
            x + w / 2.0 + offset_x,
            y + h / 2.0 + offset_y,
            5.0,
            color,
        );
    }
}

// --- Sidebar ---

fn draw_sidebar(
    state: &GameState,
    my_id: u8,
    t: &Theme,
    bid_input: &str,
    event_log: &[String],
) {
    let x = SIDE_X;
    let mut y = BOARD_Y;

    // Action card
    let action_h = 180.0;
    draw_card(x, y, SIDE_W, action_h, t);
    draw_action_card(state, my_id, t, x, y, bid_input);
    y += action_h + 12.0;

    // Players card
    let players_h = 40.0 + state.players.len() as f32 * 68.0;
    draw_card(x, y, SIDE_W, players_h, t);
    draw_players_card(state, my_id, t, x, y);
    y += players_h + 12.0;

    // Event log card
    let log_h = 30.0 + 6.0 * 22.0;
    draw_card(x, y, SIDE_W, log_h, t);
    draw_event_log(event_log, t, x, y);
}

fn draw_action_card(state: &GameState, my_id: u8, t: &Theme, x: f32, y: f32, bid_input: &str) {
    let current = &state.players[state.current_player_index];
    let is_my_turn = current.id == my_id;

    // Card header
    let header_color = if is_my_turn { t.money_color } else { t.panel_subtext };
    let header = if is_my_turn { "YOUR TURN" } else { &format!("{}'S TURN", current.name.to_uppercase()) };
    draw_text(header, x + 16.0, y + 28.0, t.label_size, header_color);
    draw_line(x + 12.0, y + 36.0, x + SIDE_W - 12.0, y + 36.0, 1.0,
        Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.4));

    // Dice display
    if let Some((d1, d2)) = state.last_roll {
        draw_text(
            &format!("Rolled: {} + {} = {}", d1, d2, d1 + d2),
            x + 16.0, y + 60.0, t.small_size, t.panel_subtext,
        );
    }

    // Auction — all players
    if let TurnPhase::Auction { highest_bid, highest_bidder, .. } = &state.turn_phase {
        draw_text("AUCTION", x + 16.0, y + 80.0, t.label_size, t.action_key_color);
        draw_text(
            &format!("Top bid: ${}", highest_bid),
            x + 16.0, y + 102.0, t.body_size, t.money_color,
        );
        draw_text(
            &format!("Leader: Player {}", highest_bidder),
            x + 16.0, y + 122.0, t.small_size, t.panel_subtext,
        );
        draw_text("Type + ENTER to bid  |  [P] Pass", x + 16.0, y + 144.0, t.small_size, t.action_key_color);

        // Bid input box
        draw_rectangle(x + 16.0, y + 152.0, 180.0, 22.0, t.input_bg);
        draw_rectangle_lines(x + 16.0, y + 152.0, 180.0, 22.0, 1.0, t.input_border_active);
        draw_text(&format!("${}", bid_input), x + 22.0, y + 168.0, t.small_size, t.panel_text);
        return;
    }

    if !is_my_turn {
        draw_text(
            &format!("Waiting for {}...", current.name),
            x + 16.0, y + 80.0, t.body_size, t.panel_subtext,
        );
        return;
    }

    let ay = y + 76.0;
    match &state.turn_phase {
        TurnPhase::WaitingForRoll => {
            action_hint(t, x, ay, "[SPACE]", "Roll the dice");
        }
        TurnPhase::BuyDecision { tile_index } => {
            let price = match &state.board.tiles[*tile_index] {
                Tile::Property(p) => p.price,
                Tile::Railroad(r) => r.price,
                Tile::Utility(u)  => u.price,
                _ => 0,
            };
            action_hint(t, x, ay,      "[B]", &format!("Buy  (${price})"));
            action_hint(t, x, ay + 28.0, "[D]", "Decline / Auction");
        }
        TurnPhase::PostRoll => {
            action_hint(t, x, ay, "[E]", "End turn");
            action_hint(t, x, ay + 28.0, "[T]", "Propose Trade");
        }
        TurnPhase::JailDecision => {
            action_hint(t, x, ay,      "[P]", "Pay fine ($50)");
            action_hint(t, x, ay + 28.0, "[R]", "Roll for doubles");
        }
        TurnPhase::PayingRent { amount, .. } => {
            draw_text(
                &format!("Rent due: ${}", amount),
                x + 16.0, ay, t.body_size, t.debt_color,
            );
            action_hint(t, x, ay + 28.0, "[E]", "Confirm payment");
        }
        TurnPhase::EndTurn => {
            draw_text("Ending turn...", x + 16.0, ay, t.body_size, t.panel_subtext);
        }
        TurnPhase::Auction { .. } => {}
    }
}

fn player_color(id: u8, t: &Theme) -> Color {
    t.player_colors[id as usize % 6]
}

fn action_hint(t: &Theme, x: f32, y: f32, key: &str, label: &str) {
    // Key badge
    draw_rectangle(x + 16.0, y - 14.0, 52.0, 20.0, t.action_bg);
    draw_rectangle_lines(x + 16.0, y - 14.0, 52.0, 20.0, 1.0, t.action_key_color);
    draw_text(key, x + 20.0, y, t.small_size, t.action_key_color);
    draw_text(label, x + 76.0, y, t.body_size, t.action_text_color);
}

fn draw_players_card(state: &GameState, my_id: u8, t: &Theme, x: f32, y: f32) {
    draw_text("PLAYERS", x + 16.0, y + 26.0, t.label_size, t.panel_subtext);
    draw_line(x + 12.0, y + 34.0, x + SIDE_W - 12.0, y + 34.0, 1.0,
        Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.4));

    for (i, player) in state.players.iter().enumerate() {
        let py = y + 48.0 + i as f32 * 68.0;
        let color = player_color(player.id, t);
        let is_current = i == state.current_player_index;
        let is_me = player.id == my_id;

        // Row highlight for current player
        if is_current {
            draw_rectangle(
                x + 8.0, py - 4.0, SIDE_W - 16.0, 58.0,
                Color::new(color.r, color.g, color.b, 0.08),
            );
            draw_rectangle_lines(
                x + 8.0, py - 4.0, SIDE_W - 16.0, 58.0,
                1.0,
                Color::new(color.r, color.g, color.b, 0.3),
            );
        }

        // Color dot
        draw_circle(x + 28.0, py + 12.0, 10.0,
            if player.bankrupt { t.bankrupt_color } else { color });

        // Name
        let name_suffix = if is_me { " (you)" } else { "" };
        draw_text(
            &format!("{}{}", player.name, name_suffix),
            x + 46.0, py + 16.0, t.body_size,
            if player.bankrupt { t.bankrupt_color } else { t.panel_text },
        );

        // Money
        draw_text(
            &format!("${}", player.money),
            x + 46.0, py + 36.0, t.label_size, t.money_color,
        );

        // Status badges
        let mut bx = x + 160.0;
        if player.in_jail {
            draw_rectangle(bx, py + 4.0, 52.0, 18.0, t.debt_color);
            draw_text("JAIL", bx + 6.0, py + 16.0, t.small_size, t.panel_text);
            bx += 58.0;
        }
        if player.bankrupt {
            draw_rectangle(bx, py + 4.0, 72.0, 18.0, t.bankrupt_color);
            draw_text("BANKRUPT", bx + 4.0, py + 16.0, t.small_size, t.panel_text);
        }

        // Property count
        draw_text(
            &format!("{} properties", player.properties.len()),
            x + 46.0, py + 52.0, t.small_size, t.panel_subtext,
        );
    }
}

fn draw_event_log(log: &[String], t: &Theme, x: f32, y: f32) {
    draw_text("EVENTS", x + 16.0, y + 22.0, t.label_size, t.panel_subtext);
    draw_line(x + 12.0, y + 30.0, x + SIDE_W - 12.0, y + 30.0, 1.0,
        Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.4));

    if log.is_empty() {
        draw_text("No events yet...", x + 16.0, y + 52.0, t.small_size, t.panel_subtext);
        return;
    }

    for (i, entry) in log.iter().enumerate() {
        let alpha = 0.4 + (i as f32 / log.len() as f32) * 0.6;
        draw_text(
            entry,
            x + 16.0, y + 50.0 + i as f32 * 22.0,
            t.small_size,
            Color::new(t.panel_text.r, t.panel_text.g, t.panel_text.b, alpha),
        );
    }
}

fn draw_tile_tooltip(state: &GameState, tile_index: usize, mx: f32, my: f32, t: &Theme) {
    use monopoly_core::board::Tile;

    let tile = &state.board.tiles[tile_index];

    // Build tooltip lines
    let mut lines: Vec<(String, Color)> = Vec::new();

    match tile {
        Tile::Property(p) => {
            let group_col = group_color(&p.color_group, t);
            lines.push((p.name.clone(), group_col));
            lines.push((format!("Price: ${}", p.price), t.panel_text));
            lines.push((format!("Build: ${} each", p.building_cost), t.panel_subtext));
            lines.push(("─────────────".into(), t.panel_subtext));
            lines.push((format!("Rent:       ${}", p.rent[0]), t.panel_text));
            lines.push((format!("1 house:    ${}", p.rent[1]), t.panel_text));
            lines.push((format!("2 houses:   ${}", p.rent[2]), t.panel_text));
            lines.push((format!("3 houses:   ${}", p.rent[3]), t.panel_text));
            lines.push((format!("4 houses:   ${}", p.rent[4]), t.panel_text));
            lines.push((format!("Hotel:      ${}", p.rent[5]), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));

            if p.mortgaged {
                lines.push(("MORTGAGED".into(), t.debt_color));
            } else if let Some(owner_id) = p.owner {
                let owner = state.players.iter().find(|pl| pl.id == owner_id);
                let owner_name = owner.map(|pl| pl.name.as_str()).unwrap_or("Unknown");
                let owner_col = t.player_colors[owner_id as usize % 6];
                lines.push((format!("Owner: {}", owner_name), owner_col));
                if p.houses > 0 && p.houses < 5 {
                    lines.push((format!("Houses: {}", p.houses), Color::from_rgba(0x2e, 0xcc, 0x71, 255)));
                } else if p.houses == 5 {
                    lines.push(("Hotel".into(), Color::from_rgba(0xe7, 0x4c, 0x3c, 255)));
                }
            } else {
                lines.push(("Unowned".into(), t.panel_subtext));
            }
        }

        Tile::Railroad(r) => {
            lines.push((r.name.clone(), t.money_color));
            lines.push((format!("Price: ${}", r.price), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
            lines.push(("Rent by railroads owned:".into(), t.panel_subtext));
            lines.push(("1 railroad:  $25".into(), t.panel_text));
            lines.push(("2 railroads: $50".into(), t.panel_text));
            lines.push(("3 railroads: $100".into(), t.panel_text));
            lines.push(("4 railroads: $200".into(), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
            if r.mortgaged {
                lines.push(("MORTGAGED".into(), t.debt_color));
            } else if let Some(owner_id) = r.owner {
                let owner = state.players.iter().find(|pl| pl.id == owner_id);
                let name = owner.map(|pl| pl.name.as_str()).unwrap_or("Unknown");
                lines.push((format!("Owner: {}", name), t.player_colors[owner_id as usize % 6]));
            } else {
                lines.push(("Unowned".into(), t.panel_subtext));
            }
        }

        Tile::Utility(u) => {
            lines.push((u.name.clone(), t.money_color));
            lines.push((format!("Price: ${}", u.price), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
            lines.push(("1 utility:  4x dice roll".into(), t.panel_text));
            lines.push(("2 utilities: 10x dice roll".into(), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
            if u.mortgaged {
                lines.push(("MORTGAGED".into(), t.debt_color));
            } else if let Some(owner_id) = u.owner {
                let owner = state.players.iter().find(|pl| pl.id == owner_id);
                let name = owner.map(|pl| pl.name.as_str()).unwrap_or("Unknown");
                lines.push((format!("Owner: {}", name), t.player_colors[owner_id as usize % 6]));
            } else {
                lines.push(("Unowned".into(), t.panel_subtext));
            }
        }

        Tile::Tax(tx) => {
            lines.push((tx.name.clone(), t.debt_color));
            lines.push((format!("Pay: ${}", tx.amount), t.panel_text));
        }

        Tile::Go => {
            lines.push(("GO".into(), t.money_color));
            lines.push(("Collect $200 salary".into(), t.panel_text));
            lines.push(("when passing or landing".into(), t.panel_subtext));
        }

        Tile::Jail => {
            lines.push(("JAIL".into(), t.panel_text));
            lines.push(("Just visiting — no effect".into(), t.panel_subtext));
            lines.push(("Sent here by Go To Jail".into(), t.panel_subtext));
            lines.push(("Pay $50 or roll doubles".into(), t.panel_subtext));
            lines.push(("to escape".into(), t.panel_subtext));
        }

        Tile::FreeParking => {
            lines.push(("FREE PARKING".into(), t.panel_text));
            lines.push(("No effect in standard rules".into(), t.panel_subtext));
        }

        Tile::GoToJail => {
            lines.push(("GO TO JAIL".into(), t.debt_color));
            lines.push(("Go directly to jail".into(), t.panel_text));
            lines.push(("Do not pass Go".into(), t.panel_subtext));
            lines.push(("Do not collect $200".into(), t.panel_subtext));
        }

        Tile::CommunityChest => {
            lines.push(("COMMUNITY CHEST".into(), t.money_color));
            lines.push(("Draw a Community Chest card".into(), t.panel_text));
            lines.push(("and follow its instructions".into(), t.panel_subtext));
        }

        Tile::Chance => {
            lines.push(("CHANCE".into(), t.money_color));
            lines.push(("Draw a Chance card".into(), t.panel_text));
            lines.push(("and follow its instructions".into(), t.panel_subtext));
        }
    }

    if lines.is_empty() { return; }

    // Tooltip dimensions
    let line_h = 20.0;
    let padding = 12.0;
    let tooltip_w = 220.0;
    let tooltip_h = padding * 2.0 + lines.len() as f32 * line_h;

    // Position — keep inside window
    let mut tx = mx + 16.0;
    let mut ty = my + 16.0;
    if tx + tooltip_w > screen_width() - 10.0 {
        tx = mx - tooltip_w - 16.0;
    }
    if ty + tooltip_h > screen_height() - 10.0 {
        ty = my - tooltip_h - 16.0;
    }

    // Draw card background
    draw_card(tx, ty, tooltip_w, tooltip_h, t);

    // Color strip on top for properties
    if let Tile::Property(p) = tile {
        let strip_col = group_color(&p.color_group, t);
        draw_rectangle(tx, ty, tooltip_w, 6.0, strip_col);
    }

    // Draw lines
    for (i, (text, color)) in lines.iter().enumerate() {
        draw_text(
            text,
            tx + padding,
            ty + padding + i as f32 * line_h + line_h - 4.0,
            t.small_size,
            *color,
        );
    }
}


fn draw_card_panel(state: &GameState, my_id: u8, t: &Theme, scroll: f32) {
    use monopoly_core::board::Tile;

    let player = match state.players.iter().find(|p| p.id == my_id) {
        Some(p) => p,
        None => return,
    };

    if player.properties.is_empty() {
        // Panel background
        let panel_h = 80.0;
        let panel_y = screen_height() - panel_h;
        draw_card(0.0, panel_y, screen_width(), panel_h, t);
        draw_text(
            "You own no properties yet",
            screen_width() / 2.0 - 120.0,
            panel_y + 44.0,
            t.body_size,
            t.panel_subtext,
        );
        return;
    }

    // Group properties by color
    let mut groups: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    let mut group_order: Vec<String> = Vec::new();

    for &tile_index in &player.properties {
        let group_key = match &state.board.tiles[tile_index] {
            Tile::Property(p) => format!("{:?}", p.color_group),
            Tile::Railroad(_) => "Railroad".to_string(),
            Tile::Utility(_)  => "Utility".to_string(),
            _ => continue,
        };
        if !groups.contains_key(&group_key) {
            group_order.push(group_key.clone());
        }
        groups.entry(group_key).or_default().push(tile_index);
    }

    // Panel dimensions
    let card_w = 100.0;
    let card_h = 130.0;
    let card_gap = 8.0;
    let panel_padding = 16.0;
    let panel_h = card_h + panel_padding * 2.0 + 24.0; // 24 for header
    let panel_y = screen_height() - panel_h;
    let panel_w = screen_width();

    // Panel background
    draw_card(0.0, panel_y, panel_w, panel_h, t);
    draw_text(
        "MY PROPERTIES",
        panel_padding, panel_y + 20.0,
        t.label_size, t.panel_subtext,
    );
  // Calculate total content width for scroll clamping
    let mut total_w = panel_padding;
    for group_key in &group_order {
        total_w += groups[group_key].len() as f32 * (card_w + card_gap) + card_gap * 2.0;
    }
    let max_scroll = (total_w - panel_w + panel_padding).max(0.0);

    let mut cx = panel_padding - scroll.min(max_scroll);
    let card_y = panel_y + 28.0;

    for group_key in &group_order {
        let tiles = &groups[group_key];
        for &tile_index in tiles {
            if cx + card_w > 0.0 && cx < panel_w {
                draw_mini_card(state, tile_index, cx, card_y, card_w, card_h, t);
            } 
            cx += card_w + card_gap;
        }
        // Small gap between groups
        cx += card_gap * 2.0;
    }
    if max_scroll > 0.0 {
        let scroll_pct = scroll.min(max_scroll) / max_scroll;
        let bar_w = panel_w - 32.0;
        let indicator_w = (panel_w / total_w * bar_w).max(30.0);
        let indicator_x = 16.0 + scroll_pct * (bar_w - indicator_w);
        draw_rectangle(16.0, panel_y + panel_h - 6.0, bar_w, 3.0,
            Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.3));
        draw_rectangle(indicator_x, panel_y + panel_h - 6.0, indicator_w, 3.0,
            t.panel_border);
    }
}

fn draw_mini_card(
    state: &GameState,
    tile_index: usize,
    x: f32, y: f32,
    w: f32, h: f32,
    t: &Theme,
) {
    use monopoly_core::board::Tile;

    // Card background
    draw_rectangle(x, y, w, h, t.input_bg);
    draw_rectangle_lines(x, y, w, h, 1.0, t.panel_border);

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            // Color strip
            let col = group_color(&p.color_group, t);
            draw_rectangle(x, y, w, 18.0, col);

            // Name
            let name = if p.name.len() > 10 {
                format!("{}.", &p.name[..9])
            } else {
                p.name.clone()
            };
            draw_text(&name, x + 4.0, y + 32.0, 10.0, t.panel_text);

            // Price
            draw_text(
                &format!("${}", p.price),
                x + 4.0, y + 48.0, 10.0, t.money_color,
            );

            // Current rent
            let rent = p.rent[p.houses as usize];
            draw_text(
                &format!("Rent: ${}", rent),
                x + 4.0, y + 64.0, 10.0, t.panel_text,
            );

            // Houses
            if p.houses > 0 && p.houses < 5 {
                for i in 0..p.houses {
                    draw_rectangle(
                        x + 4.0 + i as f32 * 16.0,
                        y + h - 22.0,
                        12.0, 12.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255),
                    );
                    draw_rectangle_lines(
                        x + 4.0 + i as f32 * 16.0,
                        y + h - 22.0,
                        12.0, 12.0,
                        1.0,
                        Color::from_rgba(0x00, 0x80, 0x00, 255),
                        );
                }
            } else if p.houses == 5 {
                draw_rectangle(x + 4.0, y + h - 22.0, 60.0, 12.0,
                    Color::from_rgba(0xe7, 0x4c, 0x3c, 255));
                draw_rectangle_lines(x + 4.0, y + h - 22.0, 60.0, 12.0, 1.0,
                    Color::from_rgba(0x8b, 0x00, 0x00, 255));
                draw_text("HOTEL", x + 8.0, y + h - 12.0, 9.0, WHITE); 
            }

            // Mortgaged overlay
            if p.mortgaged {
                draw_rectangle(x, y, w, h,
                    Color::new(0.0, 0.0, 0.0, 0.5));
                draw_text("MORTGAGED", x + 4.0, y + h / 2.0, 9.0, t.debt_color);
            }
        }

        Tile::Railroad(r) => {
            draw_rectangle(x, y, w, 18.0, t.panel_subtext);
            draw_text("RAILROAD", x + 4.0, y + 14.0, 9.0, t.panel_bg);
            let name = if r.name.len() > 10 {
                format!("{}.", &r.name[..9])
            } else {
                r.name.clone()
            };
            draw_text(&name, x + 4.0, y + 32.0, 10.0, t.panel_text);
            draw_text(&format!("${}", r.price), x + 4.0, y + 48.0, 10.0, t.money_color);
            if r.mortgaged {
                draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.5));
                draw_text("MORTGAGED", x + 4.0, y + h / 2.0, 9.0, t.debt_color);
            }
        }

        Tile::Utility(u) => {
            draw_rectangle(x, y, w, 18.0, t.group_light_blue);
            draw_text("UTILITY", x + 4.0, y + 14.0, 9.0, t.panel_bg);
            let name = if u.name.len() > 10 {
                format!("{}.", &u.name[..9])
            } else {
                u.name.clone()
            };
            draw_text(&name, x + 4.0, y + 32.0, 10.0, t.panel_text);
            draw_text(&format!("${}", u.price), x + 4.0, y + 48.0, 10.0, t.money_color);
            if u.mortgaged {
                draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.5));
                draw_text("MORTGAGED", x + 4.0, y + h / 2.0, 9.0, t.debt_color);
            }
        }

        _ => {}
    }
}

/// Returns the tile index of a mini card clicked in the panel.
fn clicked_card(
    state: &GameState,
    my_id: u8,
    mx: f32, my: f32,
    scroll: f32,
) -> Option<usize> {
    use monopoly_core::board::Tile;

    let player = state.players.iter().find(|p| p.id == my_id)?;

    let card_w = 100.0;
    let card_h = 130.0;
    let card_gap = 8.0;
    let panel_padding = 16.0;
    let panel_h = card_h + panel_padding * 2.0 + 24.0;
    let panel_y = screen_height() - panel_h;
    let card_y = panel_y + 28.0;

    // Must be clicking inside the panel
    if my < panel_y || my > panel_y + panel_h { return None; }

    let mut groups: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    let mut group_order: Vec<String> = Vec::new();

    for &tile_index in &player.properties {
        let group_key = match &state.board.tiles[tile_index] {
            Tile::Property(p) => format!("{:?}", p.color_group),
            Tile::Railroad(_) => "Railroad".to_string(),
            Tile::Utility(_)  => "Utility".to_string(),
            _ => continue,
        };
        if !groups.contains_key(&group_key) {
            group_order.push(group_key.clone());
        }
        groups.entry(group_key).or_default().push(tile_index);
    }

    let mut cx = panel_padding - scroll;
    for group_key in &group_order {
        for &tile_index in &groups[group_key] {
            if mx >= cx && mx <= cx + card_w && my >= card_y && my <= card_y + card_h {
                return Some(tile_index);
            }
            cx += card_w + card_gap;
        }
        cx += card_gap * 2.0;
    }

    None
}

fn draw_card_detail(state: &GameState, tile_index: usize, my_id: u8, t: &Theme) {
    use monopoly_core::board::Tile;

    // Dim background
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.6));

    let card_w = 280.0;
    let card_h = 420.0;
    let cx = screen_width() / 2.0 - card_w / 2.0;
    let cy = screen_height() / 2.0 - card_h / 2.0;

    draw_card(cx, cy, card_w, card_h, t);

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            let col = group_color(&p.color_group, t);

            // Color header
            draw_rectangle(cx, cy, card_w, 60.0, col);
            draw_text(&p.name.to_uppercase(), cx + 12.0, cy + 22.0, 16.0, WHITE);
            draw_text(
                &format!("TITLE DEED"),
                cx + 12.0, cy + 42.0, 11.0,
                Color::new(1.0, 1.0, 1.0, 0.7),
            );

            // Price
            draw_text(
                &format!("Price: ${}", p.price),
                cx + 12.0, cy + 80.0, t.body_size, t.panel_text,
            );
            draw_text(
                &format!("Build: ${} per house", p.building_cost),
                cx + 12.0, cy + 102.0, t.small_size, t.panel_subtext,
            );

            // Divider
            draw_line(cx + 12.0, cy + 116.0, cx + card_w - 12.0, cy + 116.0,
                1.0, t.panel_border);

            // Rent table
            let rows = [
                ("Rent", p.rent[0]),
                ("1 House", p.rent[1]),
                ("2 Houses", p.rent[2]),
                ("3 Houses", p.rent[3]),
                ("4 Houses", p.rent[4]),
                ("Hotel", p.rent[5]),
            ];
            for (i, (label, amount)) in rows.iter().enumerate() {
                let ry = cy + 130.0 + i as f32 * 26.0;
                let is_current = p.houses as usize == i;
                let text_col = if is_current { t.money_color } else { t.panel_text };
                if is_current {
                    draw_rectangle(cx + 8.0, ry - 14.0, card_w - 16.0, 22.0,
                        Color::new(t.money_color.r, t.money_color.g, t.money_color.b, 0.1));
                }
                draw_text(label, cx + 16.0, ry, t.small_size, text_col);
                draw_text(&format!("${}", amount), cx + card_w - 60.0, ry, t.small_size, text_col);
            }

            // Divider
            draw_line(cx + 12.0, cy + 300.0, cx + card_w - 12.0, cy + 300.0,
                1.0, t.panel_border);

            // Status
            if p.mortgaged {
                draw_text("MORTGAGED", cx + 12.0, cy + 320.0, t.body_size, t.debt_color);
                draw_text(
                    &format!("Unmortgage: ${}", (p.price / 2) + (p.price / 10)),
                    cx + 12.0, cy + 342.0, t.small_size, t.panel_subtext,
                );
            } else {
                draw_text(
                    &format!("Mortgage value: ${}", p.price / 2),
                    cx + 12.0, cy + 320.0, t.small_size, t.panel_subtext,
                );
            }

            // Action buttons — only shown if it's your property
            let is_owner = p.owner == Some(my_id);
            let is_my_turn = state.players[state.current_player_index].id == my_id;

            if is_owner && is_my_turn {
                let btn_y = cy + card_h - 52.0;

                if !p.mortgaged {
                    // Mortgage button
                    draw_rectangle(cx + 12.0, btn_y, 110.0, 34.0, t.debt_color);
                    draw_text("[M] Mortgage", cx + 16.0, btn_y + 22.0, t.small_size, WHITE);

                    // Build button
                    draw_rectangle(cx + 134.0, btn_y, 110.0, 34.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255));
                    draw_text("[B] Build", cx + 150.0, btn_y + 22.0, t.small_size, t.panel_bg);
                } else {
                    // Unmortgage button
                    draw_rectangle(cx + 12.0, btn_y, 150.0, 34.0, t.success_color);
                    draw_text("[U] Unmortgage", cx + 16.0, btn_y + 22.0, t.small_size, t.panel_bg);
                }
            }

            // Close hint
            draw_text("[ESC] Close", cx + 12.0, cy + card_h - 12.0,
                t.small_size, t.panel_subtext);
        }

        Tile::Railroad(r) => {
            draw_rectangle(cx, cy, card_w, 60.0, t.panel_subtext);
            draw_text(&r.name.to_uppercase(), cx + 12.0, cy + 22.0, 16.0, t.panel_bg);
            draw_text("RAILROAD", cx + 12.0, cy + 42.0, 11.0,
                Color::new(0.0, 0.0, 0.0, 0.6));

            draw_text(&format!("Price: ${}", r.price), cx + 12.0, cy + 80.0, t.body_size, t.panel_text);
            draw_line(cx + 12.0, cy + 100.0, cx + card_w - 12.0, cy + 100.0, 1.0, t.panel_border);

            let rows = [("1 Railroad", 25u32), ("2 Railroads", 50), ("3 Railroads", 100), ("4 Railroads", 200)];
            for (i, (label, amount)) in rows.iter().enumerate() {
                draw_text(label, cx + 16.0, cy + 120.0 + i as f32 * 26.0, t.small_size, t.panel_text);
                draw_text(&format!("${}", amount), cx + card_w - 60.0, cy + 120.0 + i as f32 * 26.0, t.small_size, t.panel_text);
            }

            if r.mortgaged {
                draw_text("MORTGAGED", cx + 12.0, cy + 240.0, t.body_size, t.debt_color);
            } else {
                draw_text(&format!("Mortgage: ${}", r.price / 2), cx + 12.0, cy + 240.0, t.small_size, t.panel_subtext);
            }

            if r.owner == Some(my_id) && state.players[state.current_player_index].id == my_id {
                let btn_y = cy + card_h - 52.0;
                if !r.mortgaged {
                    draw_rectangle(cx + 12.0, btn_y, 110.0, 34.0, t.debt_color);
                    draw_text("[M] Mortgage", cx + 16.0, btn_y + 22.0, t.small_size, WHITE);
                } else {
                    draw_rectangle(cx + 12.0, btn_y, 150.0, 34.0, t.success_color);
                    draw_text("[U] Unmortgage", cx + 16.0, btn_y + 22.0, t.small_size, t.panel_bg);
                }
            }

            draw_text("[ESC] Close", cx + 12.0, cy + card_h - 12.0, t.small_size, t.panel_subtext);
        }

        Tile::Utility(u) => {
            draw_rectangle(cx, cy, card_w, 60.0, t.group_light_blue);
            draw_text(&u.name.to_uppercase(), cx + 12.0, cy + 22.0, 16.0, t.panel_bg);
            draw_text("UTILITY", cx + 12.0, cy + 42.0, 11.0,
                Color::new(0.0, 0.0, 0.0, 0.6));

            draw_text(&format!("Price: ${}", u.price), cx + 12.0, cy + 80.0, t.body_size, t.panel_text);
            draw_line(cx + 12.0, cy + 100.0, cx + card_w - 12.0, cy + 100.0, 1.0, t.panel_border);
            draw_text("1 utility:   4x dice roll",  cx + 16.0, cy + 120.0, t.small_size, t.panel_text);
            draw_text("2 utilities: 10x dice roll", cx + 16.0, cy + 146.0, t.small_size, t.panel_text);

            if u.mortgaged {
                draw_text("MORTGAGED", cx + 12.0, cy + 180.0, t.body_size, t.debt_color);
            } else {
                draw_text(&format!("Mortgage: ${}", u.price / 2), cx + 12.0, cy + 180.0, t.small_size, t.panel_subtext);
            }

            if u.owner == Some(my_id) && state.players[state.current_player_index].id == my_id {
                let btn_y = cy + card_h - 52.0;
                if !u.mortgaged {
                    draw_rectangle(cx + 12.0, btn_y, 110.0, 34.0, t.debt_color);
                    draw_text("[M] Mortgage", cx + 16.0, btn_y + 22.0, t.small_size, WHITE);
                } else {
                    draw_rectangle(cx + 12.0, btn_y, 150.0, 34.0, t.success_color);
                    draw_text("[U] Unmortgage", cx + 16.0, btn_y + 22.0, t.small_size, t.panel_bg);
                }
            }

            draw_text("[ESC] Close", cx + 12.0, cy + card_h - 12.0, t.small_size, t.panel_subtext);
        }

        _ => {}
    }
}

fn draw_trade_screen(
    state: &GameState,
    my_id: u8,
    trade: &TradeScreenState,
    t: &Theme,
) {

    // Dim background
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.7));

    let panel_w = 860.0;
    let panel_h = 520.0;
    let px = screen_width() / 2.0 - panel_w / 2.0;
    let py = screen_height() / 2.0 - panel_h / 2.0;

    draw_card(px, py, panel_w, panel_h, t);

    // Header
    let target = state.players.iter().find(|p| p.id == trade.target_player);
    let target_name = target.map(|p| p.name.as_str()).unwrap_or("Unknown");
    draw_text("TRADE", px + 16.0, py + 30.0, t.title_size, t.money_color);
    draw_text(
        &format!("with {}", target_name),
        px + 120.0, py + 30.0, t.body_size, t.panel_subtext,
    );

    // Player switcher
    let active_players: Vec<&monopoly_core::player::Player> = state.players.iter()
        .filter(|p| p.id != my_id && !p.bankrupt)
        .collect();

    let mut bx = px + 16.0;
    draw_text("Trade with:", bx, py + 52.0, t.small_size, t.panel_subtext);
    bx += 80.0;
    for p in &active_players {
        let is_target = p.id == trade.target_player;
        let col = if is_target { t.money_color } else { t.panel_subtext };
        draw_rectangle(bx, py + 38.0, 80.0, 22.0,
            if is_target { Color::new(t.money_color.r, t.money_color.g, t.money_color.b, 0.2) }
            else { t.input_bg });
        draw_rectangle_lines(bx, py + 38.0, 80.0, 22.0, 1.0, col);
        draw_text(&p.name, bx + 4.0, py + 53.0, t.small_size, col);
        bx += 88.0;
    }
    draw_text("◄► to switch", bx + 4.0, py + 53.0, t.small_size, t.panel_subtext);

    draw_line(px + 12.0, py + 66.0, px + panel_w - 12.0, py + 66.0, 1.0, t.panel_border);

    // Two columns
    let col_w = (panel_w - 40.0) / 2.0;
    let left_x = px + 12.0;
    let right_x = px + 12.0 + col_w + 16.0;
    let col_y = py + 76.0;

    // --- YOUR OFFER ---
    draw_text("YOUR OFFER", left_x, col_y, t.label_size, t.panel_text);

    // Money input
    let offer_active = trade.active_input == TradeInput::OfferedMoney;
    let offer_col = if offer_active { t.input_border_active } else { t.input_border_inactive };
    draw_text("Money:", left_x, col_y + 30.0, t.small_size, t.panel_subtext);
    draw_rectangle(left_x + 60.0, col_y + 14.0, 100.0, 24.0, t.input_bg);
    draw_rectangle_lines(left_x + 60.0, col_y + 14.0, 100.0, 24.0, 1.5, offer_col);
    draw_text(
        &format!("${}", trade.offered_money_input),
        left_x + 66.0, col_y + 30.0, t.body_size, t.panel_text,
    );

    // Your properties
    draw_text("Properties:", left_x, col_y + 52.0, t.small_size, t.panel_subtext);
    let my_player = state.players.iter().find(|p| p.id == my_id);
    if let Some(player) = my_player {
        for (i, &tile_index) in player.properties.iter().enumerate() {
            let card_x = left_x + (i as f32 % 4.0) * 88.0;
            let card_y = col_y + 60.0 + (i as f32 / 4.0).floor() * 90.0;
            let selected = trade.offered_properties.contains(&tile_index);
            draw_mini_card_trade(state, tile_index, card_x, card_y, 80.0, 80.0, selected, t);
        }
    }

    // --- YOU REQUEST ---
    draw_text("YOU REQUEST", right_x, col_y, t.label_size, t.panel_text);

    // Money input
    let req_active = trade.active_input == TradeInput::RequestedMoney;
    let req_col = if req_active { t.input_border_active } else { t.input_border_inactive };
    draw_text("Money:", right_x, col_y + 30.0, t.small_size, t.panel_subtext);
    draw_rectangle(right_x + 60.0, col_y + 14.0, 100.0, 24.0, t.input_bg);
    draw_rectangle_lines(right_x + 60.0, col_y + 14.0, 100.0, 24.0, 1.5, req_col);
    draw_text(
        &format!("${}", trade.requested_money_input),
        right_x + 66.0, col_y + 30.0, t.body_size, t.panel_text,
    );

    // Target player properties
    draw_text("Properties:", right_x, col_y + 52.0, t.small_size, t.panel_subtext);
    let target_player = state.players.iter().find(|p| p.id == trade.target_player);
    if let Some(player) = target_player {
        for (i, &tile_index) in player.properties.iter().enumerate() {
            let card_x = right_x + (i as f32 % 4.0) * 88.0;
            let card_y = col_y + 60.0 + (i as f32 / 4.0).floor() * 90.0;
            let selected = trade.requested_properties.contains(&tile_index);
            draw_mini_card_trade(state, tile_index, card_x, card_y, 80.0, 80.0, selected, t);
        }
    }

    // Divider
    draw_line(px + 12.0, py + panel_h - 60.0, px + panel_w - 12.0, py + panel_h - 60.0,
        1.0, t.panel_border);

    // Bottom buttons
    draw_rectangle(px + 16.0, py + panel_h - 48.0, 160.0, 34.0, t.success_color);
    draw_text("[ENTER] Send Offer", px + 22.0, py + panel_h - 26.0, t.small_size, t.panel_bg);

    draw_rectangle(px + panel_w - 176.0, py + panel_h - 48.0, 160.0, 34.0, t.debt_color);
    draw_text("[ESC] Cancel", px + panel_w - 168.0, py + panel_h - 26.0, t.small_size, WHITE);

    draw_text("Tab: switch money field", px + 200.0, py + panel_h - 28.0, t.small_size, t.panel_subtext);
    draw_text("Click properties to add/remove", px + 400.0, py + panel_h - 28.0, t.small_size, t.panel_subtext);
}

fn draw_mini_card_trade(
    state: &GameState,
    tile_index: usize,
    x: f32, y: f32,
    w: f32, h: f32,
    selected: bool,
    t: &Theme,
) {
    use monopoly_core::board::Tile;

    // Selection highlight
    if selected {
        draw_rectangle(x - 3.0, y - 3.0, w + 6.0, h + 6.0,
            Color::new(t.money_color.r, t.money_color.g, t.money_color.b, 0.4));
    }

    draw_rectangle(x, y, w, h, t.input_bg);
    draw_rectangle_lines(x, y, w, h,
        if selected { 2.0 } else { 1.0 },
        if selected { t.money_color } else { t.panel_border },
    );

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            let col = group_color(&p.color_group, t);
            draw_rectangle(x, y, w, 14.0, col);
            let name: String = p.name.chars().take(8).collect();
            draw_text(&name, x + 3.0, y + 26.0, 9.0, t.panel_text);
            draw_text(&format!("${}", p.price), x + 3.0, y + 40.0, 9.0, t.money_color);
        }
        Tile::Railroad(r) => {
            draw_rectangle(x, y, w, 14.0, t.panel_subtext);
            let name: String = r.name.chars().take(8).collect();
            draw_text(&name, x + 3.0, y + 26.0, 9.0, t.panel_text);
            draw_text(&format!("${}", r.price), x + 3.0, y + 40.0, 9.0, t.money_color);
        }
        Tile::Utility(u) => {
            draw_rectangle(x, y, w, 14.0, t.group_light_blue);
            let name: String = u.name.chars().take(8).collect();
            draw_text(&name, x + 3.0, y + 26.0, 9.0, t.panel_text);
            draw_text(&format!("${}", u.price), x + 3.0, y + 40.0, 9.0, t.money_color);
        }
        _ => {}
    }
}
