use macroquad::prelude::*;
use serde::Deserialize;

mod classic;
mod midnight;
mod retro;

pub use classic::classic_theme;
pub use midnight::midnight_theme;
pub use retro::retro_theme;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,

    // --- Board ---
    pub board_bg: Color,
    pub board_border: Color,
    pub tile_bg: Color,
    pub tile_border: Color,
    pub tile_border_thickness: f32,
    pub color_strip_height: f32,

    // --- Panels ---
    pub panel_bg: Color,
    pub panel_border: Color,
    pub panel_text: Color,
    pub panel_subtext: Color,

    // --- Typography ---
    pub label_size: f32,
    pub body_size: f32,
    pub small_size: f32,
    pub title_size: f32,

    // --- Money ---
    pub money_color: Color,
    pub debt_color: Color,

    // --- Actions ---
    pub action_key_color: Color,
    pub action_text_color: Color,
    pub action_bg: Color,

    // --- Players ---
    pub player_colors: [Color; 6],
    pub current_player_highlight: Color,
    pub bankrupt_color: Color,

    // --- Property groups ---
    pub group_brown: Color,
    pub group_light_blue: Color,
    pub group_pink: Color,
    pub group_orange: Color,
    pub group_red: Color,
    pub group_yellow: Color,
    pub group_green: Color,
    pub group_dark_blue: Color,

    // --- UI elements ---
    pub input_border_active: Color,
    pub input_border_inactive: Color,
    pub input_bg: Color,
    pub button_bg: Color,
    pub button_text: Color,
    pub error_color: Color,
    pub success_color: Color,

    // --- Background ---
    pub window_bg: Color,
}
/// Draw a soft glowing card panel — layered rectangles for depth effect.
pub fn draw_card(x: f32, y: f32, w: f32, h: f32, theme: &Theme) {
    // Outer glow layer
    draw_rectangle(
        x - 2.0, y - 2.0, w + 4.0, h + 4.0,
        Color::new(
            theme.panel_border.r,
            theme.panel_border.g,
            theme.panel_border.b,
            0.15,
        ),
    );
    // Main card body
    draw_rectangle(x, y, w, h, theme.panel_bg);
    // Top highlight — simulates soft light from above
    draw_rectangle(
        x, y, w, 2.0,
        Color::new(1.0, 1.0, 1.0, 0.06),
    );
    // Border
    draw_rectangle_lines(x, y, w, h, 1.0, 
        Color::new(
            theme.panel_border.r,
            theme.panel_border.g,
            theme.panel_border.b,
            0.6,
        ),
    );
}
/// Load theme from config/ui.toml, fall back to classic if missing.
pub fn load_theme() -> Theme {
    #[derive(Deserialize)]
    struct UiConfig {
        theme: String,
    }

    let config = std::fs::read_to_string("config/ui.toml")
        .ok()
        .and_then(|s| toml::from_str::<UiConfig>(&s).ok());

    match config.as_ref().map(|c| c.theme.as_str()) {
        Some("midnight") => midnight_theme(),
        Some("retro")    => retro_theme(),
        _                => classic_theme(),
    }
}
