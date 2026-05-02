#![allow(dead_code)]

mod connection;
mod screens;
mod network_thread;
mod theme;

use macroquad::prelude::*;
use monopoly_core::network::ServerMessage;
use screens::Screen;
use theme::load_theme;

fn window_conf() -> Conf {
    Conf {
        window_title: "Monopoly".to_owned(),
        window_width: 1280,
        window_height: 800,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let theme = load_theme();

    let (net_tx, net_rx) = std::sync::mpsc::channel::<ServerMessage>();
    let (action_tx, action_rx) = std::sync::mpsc::channel::<monopoly_core::network::ClientMessage>();

    // Spawn networking on a separate OS thread with its own tokio runtime
    std::thread::spawn(move || {
        network_thread::run(net_tx, action_rx);
    });

    let mut screen = Screen::connect(action_tx, theme.clone());

    loop {
        clear_background(WHITE);
        screen.update(&net_rx);
        screen.draw();
        next_frame().await;
    }
}
