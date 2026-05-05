use macroquad::prelude::*;
use monopoly_core::state::GameState;
use monopoly_core::board::Tile;
use crate::theme::{Theme, draw_card};
use super::board::group_color;

pub fn draw_card_panel(state: &GameState, my_id: u8, t: &Theme, scroll: f32) {
    let player = match state.players.iter().find(|p| p.id == my_id) {
        Some(p) => p,
        None => return,
    };

    let card_w = 100.0;
    let card_h = 130.0;
    let card_gap = 8.0;
    let panel_padding = 16.0;
    let panel_h = card_h + panel_padding * 2.0 + 24.0;
    let panel_y = screen_height() - panel_h;
    let panel_w = screen_width();

    draw_card(0.0, panel_y, panel_w, panel_h, t);
    draw_text(
        "MY PROPERTIES",
        panel_padding, panel_y + 20.0,
        t.label_size, t.panel_subtext,
    );

    if player.properties.is_empty() {
        draw_text(
            "You own no properties yet",
            screen_width() / 2.0 - 120.0,
            panel_y + 44.0 + card_h / 2.0,
            t.body_size, t.panel_subtext,
        );
        return;
    }

    let (groups, group_order) = group_properties(state, &player.properties);

    // Calculate total width for scroll clamping
    let mut total_w = panel_padding;
    for key in &group_order {
        total_w += groups[key].len() as f32 * (card_w + card_gap) + card_gap * 2.0;
    }
    let max_scroll = (total_w - panel_w + panel_padding).max(0.0);
    let clamped_scroll = scroll.min(max_scroll);

    let card_y = panel_y + 28.0;
    let mut cx = panel_padding - clamped_scroll;

    for key in &group_order {
        for &tile_index in &groups[key] {
            if cx + card_w > 0.0 && cx < panel_w {
                draw_mini_card(state, tile_index, cx, card_y, card_w, card_h, t);
            }
            cx += card_w + card_gap;
        }
        cx += card_gap * 2.0;
    }

    // Scroll indicator
    if max_scroll > 0.0 {
        let scroll_pct = clamped_scroll / max_scroll;
        let bar_w = panel_w - 32.0;
        let indicator_w = (panel_w / total_w * bar_w).max(30.0);
        let indicator_x = 16.0 + scroll_pct * (bar_w - indicator_w);
        draw_rectangle(16.0, panel_y + panel_h - 6.0, bar_w, 3.0,
            Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.3));
        draw_rectangle(indicator_x, panel_y + panel_h - 6.0, indicator_w, 3.0,
            t.panel_border);
    }
}

pub fn clicked_card(
    state: &GameState,
    my_id: u8,
    mx: f32, my: f32,
    scroll: f32,
) -> Option<usize> {
    let player = state.players.iter().find(|p| p.id == my_id)?;

    let card_w = 100.0;
    let card_h = 130.0;
    let card_gap = 8.0;
    let panel_padding = 16.0;
    let panel_h = card_h + panel_padding * 2.0 + 24.0;
    let panel_y = screen_height() - panel_h;
    let card_y = panel_y + 28.0;

    if my < panel_y { return None; }

    let (groups, group_order) = group_properties(state, &player.properties);
    
    let panel_w = screen_width();
    let mut total_w = panel_padding;
    for key in &group_order {
        total_w += groups[key].len() as f32 * (card_w + card_gap) + card_gap * 2.0;
    }
    let max_scroll = (total_w - panel_w + panel_padding).max(0.0);
    let clamped_scroll = scroll.min(max_scroll);
    
    let mut cx = panel_padding - clamped_scroll;
    for key in &group_order {
        for &tile_index in &groups[key] {
            if mx >= cx && mx <= cx + card_w && my >= card_y && my <= card_y + card_h {
                return Some(tile_index);
            }
            cx += card_w + card_gap;
        }
        cx += card_gap * 2.0;
    }
    None
}

fn group_properties(
    state: &GameState,
    properties: &[usize],
) -> (std::collections::HashMap<String, Vec<usize>>, Vec<String>) {
    let mut groups: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    let mut group_order: Vec<String> = Vec::new();

    for &tile_index in properties {
        let key = match &state.board.tiles[tile_index] {
            Tile::Property(p) => format!("{:?}", p.color_group),
            Tile::Railroad(_) => "Railroad".to_string(),
            Tile::Utility(_)  => "Utility".to_string(),
            _ => continue,
        };
        if !groups.contains_key(&key) {
            group_order.push(key.clone());
        }
        groups.entry(key).or_default().push(tile_index);
    }

    (groups, group_order)
}

pub fn draw_mini_card(
    state: &GameState,
    tile_index: usize,
    x: f32, y: f32,
    w: f32, h: f32,
    t: &Theme,
) {
    draw_rectangle(x, y, w, h, t.input_bg);
    draw_rectangle_lines(x, y, w, h, 1.0, t.panel_border);

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            let col = group_color(&p.color_group, t);
            draw_rectangle(x, y, w, 18.0, col);
            let name: String = if p.name.len() > 10 {
                format!("{}.", &p.name[..9])
            } else {
                p.name.clone()
            };
            draw_text(&name, x + 4.0, y + 32.0, 10.0, t.panel_text);
            draw_text(&format!("${}", p.price), x + 4.0, y + 48.0, 10.0, t.money_color);
            draw_text(
                &format!("Rent: ${}", p.rent[p.houses as usize]),
                x + 4.0, y + 64.0, 10.0, t.panel_text,
            );

            if p.houses > 0 && p.houses < 5 {
                for i in 0..p.houses {
                    draw_rectangle(
                        x + 4.0 + i as f32 * 16.0, y + h - 22.0,
                        12.0, 12.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255),
                    );
                    draw_rectangle_lines(
                        x + 4.0 + i as f32 * 16.0, y + h - 22.0,
                        12.0, 12.0, 1.0,
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

            if p.mortgaged {
                draw_rectangle(x, y, w, h, Color::new(0.0, 0.0, 0.0, 0.5));
                draw_text("MORTGAGED", x + 4.0, y + h / 2.0, 9.0, t.debt_color);
            }
        }

        Tile::Railroad(r) => {
            draw_rectangle(x, y, w, 18.0, t.panel_subtext);
            draw_text("RAILROAD", x + 4.0, y + 14.0, 9.0, t.panel_bg);
            let name: String = if r.name.len() > 10 {
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
            let name: String = if u.name.len() > 10 {
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
