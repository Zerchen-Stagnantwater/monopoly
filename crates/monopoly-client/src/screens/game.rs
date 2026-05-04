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
            for house in 0..p.houses.min(4) {
                draw_rectangle(
                    x + 3.0 + house as f32 * 9.0,
                    y + h - 10.0,
                    7.0, 7.0,
                    GREEN,
                );
            }
            if p.houses == 5 {
                draw_rectangle(x + 3.0, y + h - 12.0, 14.0, 10.0, RED);
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
                        x + 4.0 + i as f32 * 14.0,
                        y + h - 20.0,
                        10.0, 10.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255),
                    );
                }
            } else if p.houses == 5 {
                draw_rectangle(x + 4.0, y + h - 20.0, 20.0, 10.0,
                    Color::from_rgba(0xe7, 0x4c, 0x3c, 255));
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
