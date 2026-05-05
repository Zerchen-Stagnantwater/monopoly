use macroquad::prelude::*;
use monopoly_core::state::{GameState, TurnPhase};
use monopoly_core::board::Tile;
use crate::theme::{Theme, draw_card};
use super::board::{SIDE_X, SIDE_W, player_color};

pub fn draw_sidebar(
    state: &GameState,
    my_id: u8,
    t: &Theme,
    bid_input: &str,
    event_log: &[String],
) {
    let x = SIDE_X;
    let mut y = 20.0;

    let action_h = 180.0;
    draw_card(x, y, SIDE_W, action_h, t);
    draw_action_card(state, my_id, t, x, y, bid_input);
    y += action_h + 12.0;

    let players_h = 40.0 + state.players.len() as f32 * 68.0;
    draw_card(x, y, SIDE_W, players_h, t);
    draw_players_card(state, my_id, t, x, y);
    y += players_h + 12.0;

    let log_h = 30.0 + 6.0 * 22.0;
    draw_card(x, y, SIDE_W, log_h, t);
    draw_event_log(event_log, t, x, y);
}

fn draw_action_card(state: &GameState, my_id: u8, t: &Theme, x: f32, y: f32, bid_input: &str) {
    let current = &state.players[state.current_player_index];
    let is_my_turn = current.id == my_id;

    let header_color = if is_my_turn { t.money_color } else { t.panel_subtext };
    let header = if is_my_turn {
        "YOUR TURN".to_string()
    } else {
        format!("{}'S TURN", current.name.to_uppercase())
    };
    draw_text(&header, x + 16.0, y + 28.0, t.label_size, header_color);
    draw_line(x + 12.0, y + 36.0, x + SIDE_W - 12.0, y + 36.0, 1.0,
        Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.4));

    if let Some((d1, d2)) = state.last_roll {
        draw_text(
            &format!("Rolled: {} + {} = {}", d1, d2, d1 + d2),
            x + 16.0, y + 60.0, t.small_size, t.panel_subtext,
        );
    }

    if let TurnPhase::Auction { highest_bid, highest_bidder, .. } = &state.turn_phase {
        draw_text("AUCTION", x + 16.0, y + 80.0, t.label_size, t.action_key_color);
        draw_text(
            &format!("Top bid: ${}", highest_bid),
            x + 16.0, y + 102.0, t.body_size, t.money_color,
        );
        draw_text(
            &format!("Leader: Player {}", highest_bidder),
            x + 16.0, y + 122.0, t.small_size, t.panel_subtext,
        );
        draw_text("Type + ENTER to bid  |  [P] Pass",
            x + 16.0, y + 144.0, t.small_size, t.action_key_color);
        draw_rectangle(x + 16.0, y + 152.0, 180.0, 22.0, t.input_bg);
        draw_rectangle_lines(x + 16.0, y + 152.0, 180.0, 22.0, 1.0, t.input_border_active);
        draw_text(&format!("${}", bid_input), x + 22.0, y + 168.0, t.small_size, t.panel_text);
        return;
    }

    if !is_my_turn {
        draw_text(
            &format!("Waiting for {}...", current.name),
            x + 16.0, y + 80.0, t.body_size, t.panel_subtext,
        );
        return;
    }

    let ay = y + 76.0;
    match &state.turn_phase {
        TurnPhase::WaitingForRoll => {
            action_hint(t, x, ay, "[SPACE]", "Roll the dice");
        }
        TurnPhase::BuyDecision { tile_index } => {
            let price = match &state.board.tiles[*tile_index] {
                Tile::Property(p) => p.price,
                Tile::Railroad(r) => r.price,
                Tile::Utility(u)  => u.price,
                _ => 0,
            };
            action_hint(t, x, ay,        "[B]", &format!("Buy  (${price})"));
            action_hint(t, x, ay + 28.0, "[D]", "Decline / Auction");
        }
        TurnPhase::PostRoll => {
            action_hint(t, x, ay,        "[E]", "End turn");
            action_hint(t, x, ay + 28.0, "[T]", "Propose trade");
        }
        TurnPhase::JailDecision => {
            action_hint(t, x, ay,        "[P]", "Pay fine ($50)");
            action_hint(t, x, ay + 28.0, "[R]", "Roll for doubles");
        }
        TurnPhase::PayingRent { amount, .. } => {
            draw_text(
                &format!("Rent due: ${}", amount),
                x + 16.0, ay, t.body_size, t.debt_color,
            );
            action_hint(t, x, ay + 28.0, "[E]", "Confirm payment");
        }
        TurnPhase::EndTurn => {
            draw_text("Ending turn...", x + 16.0, ay, t.body_size, t.panel_subtext);
        }
        TurnPhase::Auction { .. } => {}
    }
}

fn action_hint(t: &Theme, x: f32, y: f32, key: &str, label: &str) {
    draw_rectangle(x + 16.0, y - 14.0, 52.0, 20.0, t.action_bg);
    draw_rectangle_lines(x + 16.0, y - 14.0, 52.0, 20.0, 1.0, t.action_key_color);
    draw_text(key, x + 20.0, y, t.small_size, t.action_key_color);
    draw_text(label, x + 76.0, y, t.body_size, t.action_text_color);
}

fn draw_players_card(state: &GameState, my_id: u8, t: &Theme, x: f32, y: f32) {
    draw_text("PLAYERS", x + 16.0, y + 26.0, t.label_size, t.panel_subtext);
    draw_line(x + 12.0, y + 34.0, x + SIDE_W - 12.0, y + 34.0, 1.0,
        Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.4));

    for (i, player) in state.players.iter().enumerate() {
        let py = y + 48.0 + i as f32 * 68.0;
        let color = player_color(player.id, t);
        let is_current = i == state.current_player_index;
        let is_me = player.id == my_id;

        if is_current {
            draw_rectangle(
                x + 8.0, py - 4.0, SIDE_W - 16.0, 58.0,
                Color::new(color.r, color.g, color.b, 0.08),
            );
            draw_rectangle_lines(
                x + 8.0, py - 4.0, SIDE_W - 16.0, 58.0, 1.0,
                Color::new(color.r, color.g, color.b, 0.3),
            );
        }

        draw_circle(x + 28.0, py + 12.0, 10.0,
            if player.bankrupt { t.bankrupt_color } else { color });

        let name_suffix = if is_me { " (you)" } else { "" };
        draw_text(
            &format!("{}{}", player.name, name_suffix),
            x + 46.0, py + 16.0, t.body_size,
            if player.bankrupt { t.bankrupt_color } else { t.panel_text },
        );

        draw_text(
            &format!("${}", player.money),
            x + 46.0, py + 36.0, t.label_size, t.money_color,
        );

        let mut bx = x + 160.0;
        if player.in_jail {
            draw_rectangle(bx, py + 4.0, 52.0, 18.0, t.debt_color);
            draw_text("JAIL", bx + 6.0, py + 16.0, t.small_size, t.panel_text);
            bx += 58.0;
        }
        if player.bankrupt {
            draw_rectangle(bx, py + 4.0, 72.0, 18.0, t.bankrupt_color);
            draw_text("BANKRUPT", bx + 4.0, py + 16.0, t.small_size, t.panel_text);
        }

        draw_text(
            &format!("{} properties", player.properties.len()),
            x + 46.0, py + 52.0, t.small_size, t.panel_subtext,
        );
    }
}

fn draw_event_log(log: &[String], t: &Theme, x: f32, y: f32) {
    draw_text("EVENTS", x + 16.0, y + 22.0, t.label_size, t.panel_subtext);
    draw_line(x + 12.0, y + 30.0, x + SIDE_W - 12.0, y + 30.0, 1.0,
        Color::new(t.panel_border.r, t.panel_border.g, t.panel_border.b, 0.4));

    if log.is_empty() {
        draw_text("No events yet...", x + 16.0, y + 52.0, t.small_size, t.panel_subtext);
        return;
    }

    for (i, entry) in log.iter().enumerate() {
        let alpha = 0.4 + (i as f32 / log.len() as f32) * 0.6;
        draw_text(
            entry,
            x + 16.0, y + 50.0 + i as f32 * 22.0,
            t.small_size,
            Color::new(t.panel_text.r, t.panel_text.g, t.panel_text.b, alpha),
        );
    }
}
