use macroquad::prelude::*;
use crate::screens::Screen;

pub struct GameOverScreen {
    pub winner_id: u8,
    pub winner_name: String,
}

impl GameOverScreen {
    pub fn new(winner_id: u8, winner_name: String) -> Self {
        Self { winner_id, winner_name }
    }

    pub fn update(&mut self) -> Option<Screen> {
        None
    }

    pub fn draw(&self) {
        draw_text("GAME OVER", 480.0, 300.0, 60.0, BLACK);
        draw_text(
            &format!("{} wins!", self.winner_name),
            480.0, 400.0, 40.0, DARKGREEN,
        );
    }
}
