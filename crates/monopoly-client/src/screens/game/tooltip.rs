use macroquad::prelude::*;
use monopoly_core::state::GameState;
use monopoly_core::board::Tile;
use crate::theme::{Theme, draw_card};
use super::board::{tile_rect, group_color, player_color};

pub fn hovered_tile(mouse_x: f32, mouse_y: f32) -> Option<usize> {
    for i in 0..40 {
        let (x, y, w, h) = tile_rect(i);
        if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
            return Some(i);
        }
    }
    None
}

pub fn draw_tile_tooltip(state: &GameState, tile_index: usize, mx: f32, my: f32, t: &Theme) {
    let tile = &state.board.tiles[tile_index];
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

            if p.houses > 0 && p.houses < 5 {
                lines.push((format!("Houses: {}", p.houses),
                    Color::from_rgba(0x2e, 0xcc, 0x71, 255)));
            } else if p.houses == 5 {
                lines.push(("Hotel".into(),
                    Color::from_rgba(0xe7, 0x4c, 0x3c, 255)));
            }

            if p.mortgaged {
                lines.push(("MORTGAGED".into(), t.debt_color));
            } else if let Some(owner_id) = p.owner {
                let owner = state.players.iter().find(|pl| pl.id == owner_id);
                let name = owner.map(|pl| pl.name.as_str()).unwrap_or("Unknown");
                lines.push((format!("Owner: {}", name), player_color(owner_id, t)));
            } else {
                lines.push(("Unowned".into(), t.panel_subtext));
            }
        }

        Tile::Railroad(r) => {
            lines.push((r.name.clone(), t.money_color));
            lines.push((format!("Price: ${}", r.price), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
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
                lines.push((format!("Owner: {}", name), player_color(owner_id, t)));
            } else {
                lines.push(("Unowned".into(), t.panel_subtext));
            }
        }

        Tile::Utility(u) => {
            lines.push((u.name.clone(), t.money_color));
            lines.push((format!("Price: ${}", u.price), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
            lines.push(("1 utility:   4x dice roll".into(), t.panel_text));
            lines.push(("2 utilities: 10x dice roll".into(), t.panel_text));
            lines.push(("─────────────".into(), t.panel_subtext));
            if u.mortgaged {
                lines.push(("MORTGAGED".into(), t.debt_color));
            } else if let Some(owner_id) = u.owner {
                let owner = state.players.iter().find(|pl| pl.id == owner_id);
                let name = owner.map(|pl| pl.name.as_str()).unwrap_or("Unknown");
                lines.push((format!("Owner: {}", name), player_color(owner_id, t)));
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
            lines.push(("Draw a card and follow".into(), t.panel_text));
            lines.push(("its instructions".into(), t.panel_subtext));
        }

        Tile::Chance => {
            lines.push(("CHANCE".into(), t.money_color));
            lines.push(("Draw a card and follow".into(), t.panel_text));
            lines.push(("its instructions".into(), t.panel_subtext));
        }
    }

    if lines.is_empty() { return; }

    let line_h = 20.0;
    let padding = 12.0;
    let tooltip_w = 220.0;
    let tooltip_h = padding * 2.0 + lines.len() as f32 * line_h;

    let mut tx = mx + 16.0;
    let mut ty = my + 16.0;
    if tx + tooltip_w > screen_width() - 10.0 { tx = mx - tooltip_w - 16.0; }
    if ty + tooltip_h > screen_height() - 10.0 { ty = my - tooltip_h - 16.0; }

    draw_card(tx, ty, tooltip_w, tooltip_h, t);

    if let Tile::Property(p) = tile {
        let strip_col = group_color(&p.color_group, t);
        draw_rectangle(tx, ty, tooltip_w, 6.0, strip_col);
    }

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
