use macroquad::prelude::*;
use super::Theme;

pub fn retro_theme() -> Theme {
    Theme {
        name: "Retro",

        board_bg:              Color::from_rgba(0x00, 0xaa, 0x44, 255),
        board_border:          Color::from_rgba(0xff, 0xff, 0xff, 255),
        tile_bg:               Color::from_rgba(0xee, 0xee, 0xcc, 255),
        tile_border:           Color::from_rgba(0x00, 0x00, 0x00, 255),
        tile_border_thickness: 2.0,
        color_strip_height:    16.0,

        panel_bg:              Color::from_rgba(0x22, 0x22, 0x22, 255),
        panel_border:          Color::from_rgba(0xff, 0xff, 0xff, 255),
        panel_text:            Color::from_rgba(0xff, 0xff, 0xff, 255),
        panel_subtext:         Color::from_rgba(0xaa, 0xaa, 0xaa, 255),

        label_size:  16.0,
        body_size:   18.0,
        small_size:  12.0,
        title_size:  40.0,

        money_color: Color::from_rgba(0xff, 0xff, 0x00, 255),
        debt_color:  Color::from_rgba(0xff, 0x00, 0x00, 255),

        action_key_color:  Color::from_rgba(0xff, 0xff, 0x00, 255),
        action_text_color: Color::from_rgba(0xff, 0xff, 0xff, 255),
        action_bg:         Color::from_rgba(0x11, 0x11, 0x11, 255),

        player_colors: [
            Color::from_rgba(0xff, 0x00, 0x00, 255),
            Color::from_rgba(0x00, 0x00, 0xff, 255),
            Color::from_rgba(0x00, 0xff, 0x00, 255),
            Color::from_rgba(0xff, 0xff, 0x00, 255),
            Color::from_rgba(0xff, 0x00, 0xff, 255),
            Color::from_rgba(0x00, 0xff, 0xff, 255),
        ],
        current_player_highlight: Color::from_rgba(0xff, 0xff, 0x00, 255),
        bankrupt_color:           Color::from_rgba(0x44, 0x44, 0x44, 255),

        group_brown:      Color::from_rgba(0x88, 0x44, 0x00, 255),
        group_light_blue: Color::from_rgba(0x00, 0xaa, 0xff, 255),
        group_pink:       Color::from_rgba(0xff, 0x44, 0xaa, 255),
        group_orange:     Color::from_rgba(0xff, 0x88, 0x00, 255),
        group_red:        Color::from_rgba(0xff, 0x00, 0x00, 255),
        group_yellow:     Color::from_rgba(0xff, 0xff, 0x00, 255),
        group_green:      Color::from_rgba(0x00, 0xcc, 0x00, 255),
        group_dark_blue:  Color::from_rgba(0x00, 0x00, 0xcc, 255),

        input_border_active:   Color::from_rgba(0xff, 0xff, 0x00, 255),
        input_border_inactive: Color::from_rgba(0x77, 0x77, 0x77, 255),
        input_bg:              Color::from_rgba(0x11, 0x11, 0x11, 255),
        button_bg:             Color::from_rgba(0xff, 0xff, 0xff, 255),
        button_text:           Color::from_rgba(0x00, 0x00, 0x00, 255),
        error_color:           Color::from_rgba(0xff, 0x00, 0x00, 255),
        success_color:         Color::from_rgba(0x00, 0xff, 0x00, 255),

        window_bg: Color::from_rgba(0x11, 0x11, 0x11, 255),
    }
}
