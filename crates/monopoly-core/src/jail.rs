use crate::state::{GameState, TurnPhase};
use crate::dice::DiceRoll;
use crate::movement::advance_player;

#[derive(Debug, Clone)]
pub enum JailResult {
    /// Player paid the fine and is now free
    PaidFine,
    /// Player used a get out of jail free card
    UsedCard,
    /// Player rolled doubles and is now free
    RolledDoubles { roll: DiceRoll },
    /// Player failed to roll doubles, turn ends
    StillInJail { turns_served: u8 },
    /// Player was forced out after max turns — fine auto-deducted
    ForcedOut,
    /// Player couldn't afford the fine even when forced out
    Bankrupt,
}

/// Player chooses to pay the jail fine.
pub fn pay_jail_fine(state: &mut GameState, jail_fine: u32) -> JailResult {
    let player = &mut state.players[state.current_player_index];

    if player.money < jail_fine {
        return JailResult::Bankrupt;
    }

    player.money -= jail_fine;
    player.in_jail = false;
    player.jail_turns = 0;
    state.turn_phase = TurnPhase::WaitingForRoll;

    JailResult::PaidFine
}

/// Player uses a get out of jail free card.
pub fn use_jail_card(state: &mut GameState) -> JailResult {
    let player = &mut state.players[state.current_player_index];

    if player.get_out_of_jail == 0 {
        return JailResult::StillInJail { turns_served: player.jail_turns };
    }

    player.get_out_of_jail -= 1;
    player.in_jail = false;
    player.jail_turns = 0;
    state.turn_phase = TurnPhase::WaitingForRoll;

    JailResult::UsedCard
}

/// Player attempts to roll doubles to escape jail.
pub fn roll_for_jail(
    state: &mut GameState,
    jail_fine: u32,
    max_jail_turns: u8,
    go_salary: u32,
) -> JailResult {
    let roll = DiceRoll::roll();

    if roll.is_doubles() {
        {
            let player = &mut state.players[state.current_player_index];
            player.in_jail = false;
            player.jail_turns = 0;
        }
        let roll_clone = roll.clone();
        advance_player(state, &roll_clone, go_salary);
        return JailResult::RolledDoubles { roll };
    }

    state.players[state.current_player_index].jail_turns += 1;
    let jail_turns = state.players[state.current_player_index].jail_turns;

    if jail_turns >= max_jail_turns {
        if state.players[state.current_player_index].money >= jail_fine {
            state.players[state.current_player_index].money -= jail_fine;
            state.players[state.current_player_index].in_jail = false;
            state.players[state.current_player_index].jail_turns = 0;
            let roll_clone = roll.clone();
            advance_player(state, &roll_clone, go_salary);
            return JailResult::ForcedOut;
        } else {
            return JailResult::Bankrupt;
        }
    }

    state.turn_phase = TurnPhase::EndTurn;
    JailResult::StillInJail { turns_served: jail_turns }
}
