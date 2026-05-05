use macroquad::prelude::*;
use monopoly_core::state::GameState;
use monopoly_core::board::Tile;
use crate::theme::{Theme, draw_card};
use super::board::group_color;

pub fn draw_card_detail(state: &GameState, tile_index: usize, my_id: u8, t: &Theme) {
    // Dim background
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.6));

    let card_w = 280.0;
    let card_h = 420.0;
    let cx = screen_width() / 2.0 - card_w / 2.0;
    let cy = screen_height() / 2.0 - card_h / 2.0;

    draw_card(cx, cy, card_w, card_h, t);

    let is_my_turn = state.players[state.current_player_index].id == my_id;

    match &state.board.tiles[tile_index] {
        Tile::Property(p) => {
            let col = group_color(&p.color_group, t);

            draw_rectangle(cx, cy, card_w, 60.0, col);
            draw_text(&p.name.to_uppercase(), cx + 12.0, cy + 22.0, 16.0, WHITE);
            draw_text("TITLE DEED", cx + 12.0, cy + 42.0, 11.0,
                Color::new(1.0, 1.0, 1.0, 0.7));

            draw_text(&format!("Price: ${}", p.price),
                cx + 12.0, cy + 80.0, t.body_size, t.panel_text);
            draw_text(&format!("Build: ${} per house", p.building_cost),
                cx + 12.0, cy + 102.0, t.small_size, t.panel_subtext);

            draw_line(cx + 12.0, cy + 116.0, cx + card_w - 12.0, cy + 116.0,
                1.0, t.panel_border);

            let rows = [
                ("Rent",     p.rent[0]),
                ("1 House",  p.rent[1]),
                ("2 Houses", p.rent[2]),
                ("3 Houses", p.rent[3]),
                ("4 Houses", p.rent[4]),
                ("Hotel",    p.rent[5]),
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

            draw_line(cx + 12.0, cy + 300.0, cx + card_w - 12.0, cy + 300.0,
                1.0, t.panel_border);

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

            if p.owner == Some(my_id) && is_my_turn {
                let btn_y = cy + card_h - 52.0;
                if !p.mortgaged {
                    draw_rectangle(cx + 12.0, btn_y, 110.0, 34.0, t.debt_color);
                    draw_text("[M] Mortgage", cx + 16.0, btn_y + 22.0, t.small_size, WHITE);
                    draw_rectangle(cx + 134.0, btn_y, 110.0, 34.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255));
                    draw_text("[B] Build", cx + 150.0, btn_y + 22.0, t.small_size, t.panel_bg);
                } else {
                    draw_rectangle(cx + 12.0, btn_y, 150.0, 34.0, t.success_color);
                    draw_text("[U] Unmortgage", cx + 16.0, btn_y + 22.0, t.small_size, t.panel_bg);
                }
            }

            draw_text("[ESC] Close", cx + 12.0, cy + card_h - 12.0,
                t.small_size, t.panel_subtext);
        }

        Tile::Railroad(r) => {
            draw_rectangle(cx, cy, card_w, 60.0, t.panel_subtext);
            draw_text(&r.name.to_uppercase(), cx + 12.0, cy + 22.0, 16.0, t.panel_bg);
            draw_text("RAILROAD", cx + 12.0, cy + 42.0, 11.0,
                Color::new(0.0, 0.0, 0.0, 0.6));

            draw_text(&format!("Price: ${}", r.price),
                cx + 12.0, cy + 80.0, t.body_size, t.panel_text);
            draw_line(cx + 12.0, cy + 100.0, cx + card_w - 12.0, cy + 100.0,
                1.0, t.panel_border);

            let rows = [
                ("1 Railroad",  25u32),
                ("2 Railroads", 50),
                ("3 Railroads", 100),
                ("4 Railroads", 200),
            ];
            for (i, (label, amount)) in rows.iter().enumerate() {
                draw_text(label, cx + 16.0, cy + 120.0 + i as f32 * 26.0,
                    t.small_size, t.panel_text);
                draw_text(&format!("${}", amount), cx + card_w - 60.0,
                    cy + 120.0 + i as f32 * 26.0, t.small_size, t.panel_text);
            }

            if r.mortgaged {
                draw_text("MORTGAGED", cx + 12.0, cy + 240.0, t.body_size, t.debt_color);
            } else {
                draw_text(&format!("Mortgage: ${}", r.price / 2),
                    cx + 12.0, cy + 240.0, t.small_size, t.panel_subtext);
            }

            if r.owner == Some(my_id) && is_my_turn {
                let btn_y = cy + card_h - 52.0;
                if !r.mortgaged {
                    draw_rectangle(cx + 12.0, btn_y, 110.0, 34.0, t.debt_color);
                    draw_text("[M] Mortgage", cx + 16.0, btn_y + 22.0, t.small_size, WHITE);
                } else {
                    draw_rectangle(cx + 12.0, btn_y, 150.0, 34.0, t.success_color);
                    draw_text("[U] Unmortgage", cx + 16.0, btn_y + 22.0,
                        t.small_size, t.panel_bg);
                }
            }

            draw_text("[ESC] Close", cx + 12.0, cy + card_h - 12.0,
                t.small_size, t.panel_subtext);
        }

        Tile::Utility(u) => {
            draw_rectangle(cx, cy, card_w, 60.0, t.group_light_blue);
            draw_text(&u.name.to_uppercase(), cx + 12.0, cy + 22.0, 16.0, t.panel_bg);
            draw_text("UTILITY", cx + 12.0, cy + 42.0, 11.0,
                Color::new(0.0, 0.0, 0.0, 0.6));

            draw_text(&format!("Price: ${}", u.price),
                cx + 12.0, cy + 80.0, t.body_size, t.panel_text);
            draw_line(cx + 12.0, cy + 100.0, cx + card_w - 12.0, cy + 100.0,
                1.0, t.panel_border);
            draw_text("1 utility:   4x dice roll",
                cx + 16.0, cy + 120.0, t.small_size, t.panel_text);
            draw_text("2 utilities: 10x dice roll",
                cx + 16.0, cy + 146.0, t.small_size, t.panel_text);

            if u.mortgaged {
                draw_text("MORTGAGED", cx + 12.0, cy + 180.0, t.body_size, t.debt_color);
            } else {
                draw_text(&format!("Mortgage: ${}", u.price / 2),
                    cx + 12.0, cy + 180.0, t.small_size, t.panel_subtext);
            }

            if u.owner == Some(my_id) && is_my_turn {
                let btn_y = cy + card_h - 52.0;
                if !u.mortgaged {
                    draw_rectangle(cx + 12.0, btn_y, 110.0, 34.0, t.debt_color);
                    draw_text("[M] Mortgage", cx + 16.0, btn_y + 22.0, t.small_size, WHITE);
                } else {
                    draw_rectangle(cx + 12.0, btn_y, 150.0, 34.0, t.success_color);
                    draw_text("[U] Unmortgage", cx + 16.0, btn_y + 22.0,
                        t.small_size, t.panel_bg);
                }
            }
            draw_text("[ESC] Close", cx + 12.0, cy + card_h - 12.0,
                t.small_size, t.panel_subtext);
        }

        _ => {}
    }
}
