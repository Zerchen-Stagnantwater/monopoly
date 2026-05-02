use macroquad::prelude::*;
use super::Theme;

pub fn midnight_theme() -> Theme {
    Theme {
        name: "Midnight",

        board_bg:              Color::from_rgba(0x0d, 0x0d, 0x1a, 255),
        board_border:          Color::from_rgba(0x00, 0xff, 0xff, 255), // cyan
        tile_bg:               Color::from_rgba(0x11, 0x11, 0x22, 255),
        tile_border:           Color::from_rgba(0x33, 0x33, 0x66, 255),
        tile_border_thickness: 1.0,
        color_strip_height:    12.0,

        panel_bg:              Color::from_rgba(0x05, 0x05, 0x10, 255),
        panel_border:          Color::from_rgba(0x00, 0xff, 0xff, 100),
        panel_text:            Color::from_rgba(0xe0, 0xe0, 0xff, 255),
        panel_subtext:         Color::from_rgba(0x77, 0x77, 0xaa, 255),

        label_size:  17.0,
        body_size:   19.0,
        small_size:  13.0,
        title_size:  38.0,

        money_color: Color::from_rgba(0x00, 0xff, 0x99, 255), // neon green
        debt_color:  Color::from_rgba(0xff, 0x22, 0x55, 255),

        action_key_color:  Color::from_rgba(0x00, 0xff, 0xff, 255),
        action_text_color: Color::from_rgba(0xe0, 0xe0, 0xff, 255),
        action_bg:         Color::from_rgba(0x05, 0x05, 0x10, 255),

        player_colors: [
            Color::from_rgba(0xff, 0x22, 0x55, 255),
            Color::from_rgba(0x00, 0xcc, 0xff, 255),
            Color::from_rgba(0x00, 0xff, 0x99, 255),
            Color::from_rgba(0xff, 0xaa, 0x00, 255),
            Color::from_rgba(0xcc, 0x44, 0xff, 255),
            Color::from_rgba(0xff, 0xff, 0x00, 255),
        ],
        current_player_highlight: Color::from_rgba(0x00, 0xff, 0xff, 255),
        bankrupt_color:           Color::from_rgba(0x33, 0x33, 0x44, 255),

        group_brown:      Color::from_rgba(0xaa, 0x55, 0x22, 255),
        group_light_blue: Color::from_rgba(0x00, 0xcc, 0xff, 255),
        group_pink:       Color::from_rgba(0xff, 0x44, 0xaa, 255),
        group_orange:     Color::from_rgba(0xff, 0x88, 0x00, 255),
        group_red:        Color::from_rgba(0xff, 0x22, 0x22, 255),
        group_yellow:     Color::from_rgba(0xff, 0xff, 0x00, 255),
        group_green:      Color::from_rgba(0x00, 0xff, 0x55, 255),
        group_dark_blue:  Color::from_rgba(0x44, 0x44, 0xff, 255),

        input_border_active:   Color::from_rgba(0x00, 0xff, 0xff, 255),
        input_border_inactive: Color::from_rgba(0x33, 0x33, 0x66, 255),
        input_bg:              Color::from_rgba(0x05, 0x05, 0x10, 255),
        button_bg:             Color::from_rgba(0x00, 0xff, 0xff, 255),
        button_text:           Color::from_rgba(0x05, 0x05, 0x10, 255),
        error_color:           Color::from_rgba(0xff, 0x22, 0x55, 255),
        success_color:         Color::from_rgba(0x00, 0xff, 0x99, 255),

        window_bg: Color::from_rgba(0x03, 0x03, 0x08, 255),
    }
}
