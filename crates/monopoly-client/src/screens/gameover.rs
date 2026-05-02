use macroquad::prelude::*;
use crate::screens::Screen;
use crate::theme::Theme;

pub struct GameOverScreen {
    pub winner_id: u8,
    pub winner_name: String,
    pub theme: Theme,
}

impl GameOverScreen {
    pub fn new(winner_id: u8, winner_name: String, theme: Theme) -> Self {
        Self { winner_id, winner_name, theme }
    }

    pub fn update(&mut self) -> Option<Screen> {
        None
    }

    pub fn draw(&self) {
        let t = &self.theme;
        let cx = screen_width() / 2.0;
        let cy = screen_height() / 2.0;

        draw_rectangle(cx - 300.0, cy - 150.0, 600.0, 300.0, t.panel_bg);
        draw_rectangle_lines(cx - 300.0, cy - 150.0, 600.0, 300.0, 2.0, t.panel_border);

        draw_text("GAME OVER", cx - 140.0, cy - 60.0, t.title_size, t.money_color);
        draw_text(
            &format!("{} WINS!", self.winner_name.to_uppercase()),
            cx - 150.0, cy, t.body_size * 1.5, t.success_color,
        );
        draw_text("Thanks for playing", cx - 100.0, cy + 60.0, t.label_size, t.panel_subtext);
    }
}
