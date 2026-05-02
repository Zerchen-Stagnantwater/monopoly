use macroquad::prelude::*;
use super::Theme;

pub fn classic_theme() -> Theme {
    Theme {
        name: "Classic",

        board_bg:              Color::from_rgba(0xce, 0xe8, 0xd0, 255), // classic green
        board_border:          Color::from_rgba(0x00, 0x00, 0x00, 255),
        tile_bg:               Color::from_rgba(0xf5, 0xf0, 0xe8, 255), // cream
        tile_border:           Color::from_rgba(0x00, 0x00, 0x00, 255),
        tile_border_thickness: 1.5,
        color_strip_height:    14.0,

        panel_bg:              Color::from_rgba(0x1a, 0x1a, 0x2e, 255), // deep navy
        panel_border:          Color::from_rgba(0x2a, 0x2a, 0x4e, 255),
        panel_text:            Color::from_rgba(0xff, 0xff, 0xff, 255),
        panel_subtext:         Color::from_rgba(0xaa, 0xaa, 0xcc, 255),

        label_size:  18.0,
        body_size:   20.0,
        small_size:  14.0,
        title_size:  36.0,

        money_color: Color::from_rgba(0xf0, 0xc0, 0x40, 255), // gold
        debt_color:  Color::from_rgba(0xff, 0x44, 0x44, 255),

        action_key_color:  Color::from_rgba(0xf0, 0xc0, 0x40, 255),
        action_text_color: Color::from_rgba(0xff, 0xff, 0xff, 255),
        action_bg:         Color::from_rgba(0x0f, 0x0f, 0x1e, 255),

        player_colors: [
            Color::from_rgba(0xe7, 0x4c, 0x3c, 255), // red
            Color::from_rgba(0x34, 0x98, 0xdb, 255), // blue
            Color::from_rgba(0x2e, 0xcc, 0x71, 255), // green
            Color::from_rgba(0xf3, 0x9c, 0x12, 255), // orange
            Color::from_rgba(0x9b, 0x59, 0xb6, 255), // purple
            Color::from_rgba(0x1a, 0xbc, 0x9c, 255), // teal
        ],
        current_player_highlight: Color::from_rgba(0xf0, 0xc0, 0x40, 255),
        bankrupt_color:           Color::from_rgba(0x55, 0x55, 0x55, 255),

        group_brown:      Color::from_rgba(0x8b, 0x45, 0x13, 255),
        group_light_blue: Color::from_rgba(0x87, 0xce, 0xeb, 255),
        group_pink:       Color::from_rgba(0xff, 0x69, 0xb4, 255),
        group_orange:     Color::from_rgba(0xff, 0xa5, 0x00, 255),
        group_red:        Color::from_rgba(0xdc, 0x14, 0x3c, 255),
        group_yellow:     Color::from_rgba(0xff, 0xd7, 0x00, 255),
        group_green:      Color::from_rgba(0x00, 0x80, 0x00, 255),
        group_dark_blue:  Color::from_rgba(0x00, 0x00, 0x8b, 255),

        input_border_active:   Color::from_rgba(0xf0, 0xc0, 0x40, 255),
        input_border_inactive: Color::from_rgba(0x55, 0x55, 0x77, 255),
        input_bg:              Color::from_rgba(0x0f, 0x0f, 0x1e, 255),
        button_bg:             Color::from_rgba(0xf0, 0xc0, 0x40, 255),
        button_text:           Color::from_rgba(0x1a, 0x1a, 0x2e, 255),
        error_color:           Color::from_rgba(0xff, 0x44, 0x44, 255),
        success_color:         Color::from_rgba(0x2e, 0xcc, 0x71, 255),

        window_bg: Color::from_rgba(0x12, 0x12, 0x20, 255),
    }
}
