use macroquad::prelude::*;
use monopoly_core::state::GameState;
use monopoly_core::board::Tile;
use crate::theme::{Theme, draw_card};
use super::board::group_color;

#[derive(Debug, Clone, PartialEq)]
pub enum TradeInput {
    OfferedMoney,
    RequestedMoney,
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

pub fn draw_trade_screen(
    state: &GameState,
    my_id: u8,
    trade: &TradeScreenState,
    t: &Theme,
) {
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
            if is_target {
                Color::new(t.money_color.r, t.money_color.g, t.money_color.b, 0.2)
            } else {
                t.input_bg
            });
        draw_rectangle_lines(bx, py + 38.0, 80.0, 22.0, 1.0, col);
        draw_text(&p.name, bx + 4.0, py + 53.0, t.small_size, col);
        bx += 88.0;
    }
    draw_text("◄► to switch", bx + 4.0, py + 53.0, t.small_size, t.panel_subtext);

    draw_line(px + 12.0, py + 66.0, px + panel_w - 12.0, py + 66.0,
        1.0, t.panel_border);

    let col_w = (panel_w - 40.0) / 2.0;
    let left_x  = px + 12.0;
    let right_x = px + 12.0 + col_w + 16.0;
    let col_y   = py + 76.0;

    // --- YOUR OFFER ---
    draw_text("YOUR OFFER", left_x, col_y, t.label_size, t.panel_text);

    let offer_col = if trade.active_input == TradeInput::OfferedMoney {
        t.input_border_active
    } else {
        t.input_border_inactive
    };
    draw_text("Money:", left_x, col_y + 30.0, t.small_size, t.panel_subtext);
    draw_rectangle(left_x + 60.0, col_y + 14.0, 100.0, 24.0, t.input_bg);
    draw_rectangle_lines(left_x + 60.0, col_y + 14.0, 100.0, 24.0, 1.5, offer_col);
    draw_text(
        &format!("${}", trade.offered_money_input),
        left_x + 66.0, col_y + 30.0, t.body_size, t.panel_text,
    );

    draw_text("Properties:", left_x, col_y + 52.0, t.small_size, t.panel_subtext);
    if let Some(player) = state.players.iter().find(|p| p.id == my_id) {
        for (i, &tile_index) in player.properties.iter().enumerate() {
            let card_x = left_x + (i as f32 % 4.0) * 88.0;
            let card_y = col_y + 60.0 + (i as f32 / 4.0).floor() * 90.0;
            let selected = trade.offered_properties.contains(&tile_index);
            draw_mini_card_trade(state, tile_index, card_x, card_y, 80.0, 80.0, selected, t);
        }
    }

    // --- YOU REQUEST ---
    draw_text("YOU REQUEST", right_x, col_y, t.label_size, t.panel_text);

    let req_col = if trade.active_input == TradeInput::RequestedMoney {
        t.input_border_active
    } else {
        t.input_border_inactive
    };
    draw_text("Money:", right_x, col_y + 30.0, t.small_size, t.panel_subtext);
    draw_rectangle(right_x + 60.0, col_y + 14.0, 100.0, 24.0, t.input_bg);
    draw_rectangle_lines(right_x + 60.0, col_y + 14.0, 100.0, 24.0, 1.5, req_col);
    draw_text(
        &format!("${}", trade.requested_money_input),
        right_x + 66.0, col_y + 30.0, t.body_size, t.panel_text,
    );

    draw_text("Properties:", right_x, col_y + 52.0, t.small_size, t.panel_subtext);
    if let Some(player) = state.players.iter().find(|p| p.id == trade.target_player) {
        for (i, &tile_index) in player.properties.iter().enumerate() {
            let card_x = right_x + (i as f32 % 4.0) * 88.0;
            let card_y = col_y + 60.0 + (i as f32 / 4.0).floor() * 90.0;
            let selected = trade.requested_properties.contains(&tile_index);
            draw_mini_card_trade(state, tile_index, card_x, card_y, 80.0, 80.0, selected, t);
        }
    }

    // Bottom bar
    draw_line(px + 12.0, py + panel_h - 60.0, px + panel_w - 12.0, py + panel_h - 60.0,
        1.0, t.panel_border);

    draw_rectangle(px + 16.0, py + panel_h - 48.0, 160.0, 34.0, t.success_color);
    draw_text("[ENTER] Send Offer", px + 22.0, py + panel_h - 26.0, t.small_size, t.panel_bg);

    draw_rectangle(px + panel_w - 176.0, py + panel_h - 48.0, 160.0, 34.0, t.debt_color);
    draw_text("[ESC] Cancel", px + panel_w - 168.0, py + panel_h - 26.0, t.small_size, WHITE);

    draw_text("Tab: switch money field", px + 200.0, py + panel_h - 28.0,
        t.small_size, t.panel_subtext);
    draw_text("Click properties to add/remove", px + 400.0, py + panel_h - 28.0,
        t.small_size, t.panel_subtext);
}

fn draw_mini_card_trade(
    state: &GameState,
    tile_index: usize,
    x: f32, y: f32,
    w: f32, h: f32,
    selected: bool,
    t: &Theme,
) {
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
