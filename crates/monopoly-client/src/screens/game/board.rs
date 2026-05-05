use macroquad::prelude::*;
use monopoly_core::state::GameState;
use monopoly_core::board::{Tile, ColorGroup};
use crate::theme::Theme;

// Board geometry — shared constants
pub const BOARD_X: f32 = 20.0;
pub const BOARD_Y: f32 = 20.0;
pub const BOARD_SIZE: f32 = 680.0;
pub const CORNER_SIZE: f32 = 76.0;
pub const TILE_W: f32 = (BOARD_SIZE - 2.0 * CORNER_SIZE) / 9.0;
pub const TILE_H: f32 = CORNER_SIZE;
pub const SIDE_X: f32 = BOARD_X + BOARD_SIZE + 16.0;
pub const SIDE_W: f32 = 340.0;

pub fn group_color(group: &ColorGroup, t: &Theme) -> Color {
    match group {
        ColorGroup::Brown     => t.group_brown,
        ColorGroup::LightBlue => t.group_light_blue,
        ColorGroup::Pink      => t.group_pink,
        ColorGroup::Orange    => t.group_orange,
        ColorGroup::Red       => t.group_red,
        ColorGroup::Yellow    => t.group_yellow,
        ColorGroup::Green     => t.group_green,
        ColorGroup::DarkBlue  => t.group_dark_blue,
    }
}

pub fn tile_rect(index: usize) -> (f32, f32, f32, f32) {
    let b = BOARD_X;
    let t = BOARD_Y;
    let s = BOARD_SIZE;
    let c = CORNER_SIZE;
    let tw = TILE_W;
    let th = TILE_H;

    match index {
        0  => (b + s - c, t + s - c, c, c),
        1  => (b + s - c - tw,      t + s - th, tw, th),
        2  => (b + s - c - 2.0*tw,  t + s - th, tw, th),
        3  => (b + s - c - 3.0*tw,  t + s - th, tw, th),
        4  => (b + s - c - 4.0*tw,  t + s - th, tw, th),
        5  => (b + s - c - 5.0*tw,  t + s - th, tw, th),
        6  => (b + s - c - 6.0*tw,  t + s - th, tw, th),
        7  => (b + s - c - 7.0*tw,  t + s - th, tw, th),
        8  => (b + s - c - 8.0*tw,  t + s - th, tw, th),
        9  => (b + s - c - 9.0*tw,  t + s - th, tw, th),
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
        21 => (b + c,          t, tw, th),
        22 => (b + c + tw,     t, tw, th),
        23 => (b + c + 2.0*tw, t, tw, th),
        24 => (b + c + 3.0*tw, t, tw, th),
        25 => (b + c + 4.0*tw, t, tw, th),
        26 => (b + c + 5.0*tw, t, tw, th),
        27 => (b + c + 6.0*tw, t, tw, th),
        28 => (b + c + 7.0*tw, t, tw, th),
        29 => (b + c + 8.0*tw, t, tw, th),
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

pub fn draw_board(state: &GameState, t: &Theme) {
    draw_rectangle(BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE, t.board_bg);
    draw_rectangle_lines(BOARD_X, BOARD_Y, BOARD_SIZE, BOARD_SIZE, 2.0, t.board_border);

    for (i, tile) in state.board.tiles.iter().enumerate() {
        let (x, y, w, h) = tile_rect(i);

        draw_rectangle(x, y, w, h, t.tile_bg);
        draw_rectangle_lines(x, y, w, h, t.tile_border_thickness, t.tile_border);

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

            if let Some(owner_id) = p.owner {
                let oc = player_color(owner_id, t);
                draw_circle(x + w - 8.0, y + 8.0, 5.0, oc);
            }

            if p.houses > 0 && p.houses < 5 {
                for house in 0..p.houses {
                    draw_rectangle(
                        x + 2.0 + house as f32 * 8.0,
                        y + h - 11.0, 6.0, 6.0,
                        Color::from_rgba(0x2e, 0xcc, 0x71, 255),
                    );
                    draw_rectangle_lines(
                        x + 2.0 + house as f32 * 8.0,
                        y + h - 11.0, 6.0, 6.0, 1.0,
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

        let label = tile_label(tile);
        draw_text(&label, x + 2.0, y + h / 2.0 + 4.0, 9.0, t.tile_border);
    }

    let cx = BOARD_X + BOARD_SIZE / 2.0;
    let cy = BOARD_Y + BOARD_SIZE / 2.0;
    draw_text("MONOPOLY", cx - 55.0, cy - 8.0, 26.0, t.board_border);
    draw_text(
        &format!("Turn {}", 0),
        cx - 30.0, cy + 18.0, 14.0,
        Color::new(t.board_border.r, t.board_border.g, t.board_border.b, 0.5),
    );
}

pub fn draw_players(state: &GameState, t: &Theme) {
    for (pi, player) in state.players.iter().enumerate() {
        if player.bankrupt { continue; }
        let (x, y, w, h) = tile_rect(player.position);
        let color = player_color(player.id, t);
        let offset_x = (pi as f32 % 3.0) * 12.0 - 12.0;
        let offset_y = (pi as f32 / 3.0).floor() * 12.0 - 6.0;

        draw_circle(
            x + w / 2.0 + offset_x,
            y + h / 2.0 + offset_y,
            7.0,
            Color::new(color.r, color.g, color.b, 0.3),
        );
        draw_circle(
            x + w / 2.0 + offset_x,
            y + h / 2.0 + offset_y,
            5.0, color,
        );
    }
}

pub fn player_color(id: u8, t: &Theme) -> Color {
    t.player_colors[id as usize % 6]
}
