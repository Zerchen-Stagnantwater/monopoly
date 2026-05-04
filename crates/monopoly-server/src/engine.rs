use anyhow::{Result, bail};
use tokio::sync::Mutex;
use std::sync::Arc;
use monopoly_core::network::{ClientMessage, ServerMessage};
use monopoly_core::state::{GameState, TurnPhase};
use monopoly_core::{
    advance_player, buy_property, decline_purchase, place_bid, finalize_auction,
    pay_jail_fine, use_jail_card, roll_for_jail,
    build_house, sell_house, mortgage_property, unmortgage_property,
    declare_bankrupt, DiceRoll,
};
use monopoly_core::trading::{execute_trade, TradeOffer};
use crate::lobby::{Lobby, ClientTx};

pub async fn handle_message(
    lobby: &Arc<Mutex<Lobby>>,
    player_id: Option<u8>,
    msg: ClientMessage,
    tx: ClientTx,
) -> Result<Option<u8>> {
    let mut lobby = lobby.lock().await;

    match msg {

        ClientMessage::Join { name, token } => {
            let id = lobby.add_player(name.clone(), token.clone(), tx)
                .ok_or_else(|| anyhow::anyhow!("Lobby full or game already started"))?;

            // Tell the new player their id
            lobby.send_to(id, ServerMessage::JoinAck { assigned_id: id });

            // Send existing roster to the new player
            let roster = lobby.players.values()
                .map(|p| (p.id, p.name.clone(), p.token.clone()))
                .collect();
            lobby.send_to(id, ServerMessage::LobbyState { players: roster });

            // Tell everyone else about the new player
            // Tell everyone else about the new player (not the joiner themselves)
            for (&pid, player) in &lobby.players {
              if pid != id {
                let _ = player.tx.send(ServerMessage::PlayerJoined {
                    id,
                    name: name.clone(),
                    token: token.clone(),
                });
              }
            }
            return Ok(Some(id));
        }      

        ClientMessage::StartGame => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            if lobby.host_id != Some(id) { bail!("Only the host can start the game"); }
            if lobby.players.len() < 2 { bail!("Need at least 2 players to start"); }
            lobby.start_game()?;
            lobby.broadcast(ServerMessage::GameStarted);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::BuyProperty => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            let tile_index = match state.turn_phase {
                TurnPhase::BuyDecision { tile_index } => tile_index,
                _ => bail!("Not in buy phase"),
            };
            buy_property(state, tile_index);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::DeclineProperty => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            // Clone before mutable borrow
            let auction_enabled = lobby.ruleset.auction_enabled;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            let tile_index = match state.turn_phase {
                TurnPhase::BuyDecision { tile_index } => tile_index,
                _ => bail!("Not in buy phase"),
            };
            decline_purchase(state, tile_index, auction_enabled);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::PayRent => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;

            if state.players[state.current_player_index].id != id {
                bail!("Not your turn");
            }

            let (amount, to_player) = match state.turn_phase {
                TurnPhase::PayingRent { amount, to_player } => (amount, to_player),
                    _ => bail!("Not in rent phase"),
            };

            // Deduct from payer
            state.players[state.current_player_index].money =
            state.players[state.current_player_index].money.saturating_sub(amount);

            // Pay the owner
            if let Some(owner) = state.players.iter_mut().find(|p| p.id == to_player) {
                owner.money += amount;
            }

            state.turn_phase = TurnPhase::PostRoll;

            let log = format!(
                "{} paid ${} in rent",
                state.players[state.current_player_index].name,
                amount
                );
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::EventLog { message: log });
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::PlaceBid { amount } => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if !place_bid(state, id, amount) { bail!("Bid rejected — too low or insufficient funds"); }
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::PassBid => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;

            if !state.auction_passers.contains(&id) {
                state.auction_passers.push(id);
            }

            // Check if all active players have passed
            let active_count = state.players.iter().filter(|p| !p.bankrupt).count();
            if state.auction_passers.len() >= active_count {
                finalize_auction(state);
                state.auction_passers.clear();
            }

            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::FinalizeAuction => {
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            finalize_auction(state);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::PayJailFine => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let jail_fine = lobby.ruleset.jail_fine;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            pay_jail_fine(state, jail_fine);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::UseJailCard => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            use_jail_card(state);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::RollForJail => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            // Clone before mutable borrow
            let jail_fine = lobby.ruleset.jail_fine;
            let max_jail_turns = lobby.ruleset.max_jail_turns;
            let go_salary = lobby.ruleset.go_salary;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            roll_for_jail(state, jail_fine, max_jail_turns, go_salary);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }
      
        ClientMessage::RollDice => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let go_salary = lobby.ruleset.go_salary;
            let free_parking_pot = lobby.ruleset.free_parking_pot;

            {
                let state = lobby.game.as_mut()
                    .ok_or_else(|| anyhow::anyhow!("Game not started"))?;
                if state.players[state.current_player_index].id != id {
                    bail!("Not your turn");
                }
                if state.turn_phase != TurnPhase::WaitingForRoll {
                    bail!("Not in roll phase");
                }

                let roll = DiceRoll::roll();
                let result = advance_player(state, &roll, go_salary);

                use monopoly_core::movement::MoveResult;
                match result {
                    MoveResult::LandedOnProperty { tile_index } => {
                        state.turn_phase = TurnPhase::BuyDecision { tile_index };
                    }
                    MoveResult::LandedOnOwnedProperty { tile_index, owner: _ } => {
                        let rent = monopoly_core::calculate_rent(state, tile_index, &roll);
                        let owner = match &state.board.tiles[tile_index] {
                            monopoly_core::board::Tile::Property(p) => p.owner.unwrap(),
                            monopoly_core::board::Tile::Railroad(r) => r.owner.unwrap(),
                            monopoly_core::board::Tile::Utility(u)  => u.owner.unwrap(),
                                _ => 0,
                        };
                        state.turn_phase = TurnPhase::PayingRent { amount: rent, to_player: owner };
                    }
                    MoveResult::SentToJail => {
                        state.turn_phase = TurnPhase::JailDecision;
                    }
                    MoveResult::LandedOnTax { amount } => {
                        state.players[state.current_player_index].money =
                            state.players[state.current_player_index].money.saturating_sub(amount);
                        if free_parking_pot {
                            // handled below after borrow ends
                        }
                        state.turn_phase = TurnPhase::PostRoll;
                    }
                    MoveResult::LandedOnCommunityChest { .. } => {
                        let card = state.card_decks.draw_community_chest();
                        let desc = card.description.clone();
                        apply_card(state, card, go_salary);
                        lobby.broadcast(ServerMessage::EventLog {
                            message: format!("Community Chest: {}", desc),
                            });
                    }
                    MoveResult::LandedOnChance { .. } => {
                        let card = state.card_decks.draw_chance();
                        let desc = card.description.clone();
                        apply_card(state, card, go_salary);
                        lobby.broadcast(ServerMessage::EventLog {
                            message: format!("Chance: {}", desc),
                        });
                    }
                    _ => {
                        state.turn_phase = TurnPhase::PostRoll;
                    }
                }
            }

            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::EventLog {
                message: format!(
                    "{} rolled {:?}",
                    state.players[state.current_player_index].name,
                    state.last_roll,
                    ),
                    });
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::BuildHouse { tile_index } => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            build_house(state, tile_index);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::SellHouse { tile_index } => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            sell_house(state, tile_index);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::Mortgage { tile_index } => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            mortgage_property(state, tile_index);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::Unmortgage { tile_index } => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            unmortgage_property(state, tile_index);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::ProposeTrade {
            to_player, offered_properties, offered_money, requested_properties, requested_money,
        } => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let offer = TradeOffer {
                from_player: id,
                to_player,
                offered_properties: offered_properties.clone(),
                offered_money,
                requested_properties: requested_properties.clone(),
                requested_money,
            };
            lobby.pending_trade = Some(offer);
            lobby.send_to(to_player, ServerMessage::TradeProposed {
                from_player: id,
                offered_properties,
                offered_money,
                requested_properties,
                requested_money,
            });
        }

        ClientMessage::AcceptTrade => {
            let trade = lobby.pending_trade.take()
                .ok_or_else(|| anyhow::anyhow!("No pending trade"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            execute_trade(state, trade);
            lobby.broadcast(ServerMessage::TradeAccepted);
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::RejectTrade => {
            lobby.pending_trade = None;
            lobby.broadcast(ServerMessage::TradeRejected);
        }

        ClientMessage::EndTurn => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            if state.players[state.current_player_index].id != id { bail!("Not your turn"); }
            state.advance_turn();
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

        ClientMessage::Connect { .. } => {
            // Internal client signal, never sent over the wire
        }

        ClientMessage::DeclareBankruptcy => {
            let id = player_id.ok_or_else(|| anyhow::anyhow!("Not joined"))?;
            let state = lobby.game.as_mut().ok_or_else(|| anyhow::anyhow!("Game not started"))?;
            let result = declare_bankrupt(state, id, None);
            use monopoly_core::bankruptcy::BankruptcyResult;
            if let BankruptcyResult::GameOver { winner } = result {
                let winner_name = state.players.iter()
                    .find(|p| p.id == winner)
                    .map(|p| p.name.clone())
                    .unwrap_or_default();
                lobby.broadcast(ServerMessage::GameOver { winner_id: winner, winner_name });
            }
            let state = lobby.game.clone().unwrap();
            lobby.broadcast(ServerMessage::StateUpdate { state });
        }

    }

    Ok(None)
}

/// Apply a drawn card's effect to the current player.
fn apply_card(state: &mut GameState, card: monopoly_core::cards::Card, go_salary: u32) {
    use monopoly_core::cards::CardEffect;
    use monopoly_core::board::Tile;

    state.last_card = Some(card.description.clone());

    match card.effect {
        CardEffect::Collect { amount } => {
            state.players[state.current_player_index].money += amount;
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::Pay { amount } => {
            state.players[state.current_player_index].money =
                state.players[state.current_player_index].money.saturating_sub(amount);
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::MoveTo { tile_index } => {
            let old_pos = state.players[state.current_player_index].position;
            state.players[state.current_player_index].position = tile_index;
            // Collect Go salary if passed Go
            if tile_index < old_pos {
                state.players[state.current_player_index].money += go_salary;
            }
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::MoveBack { spaces } => {
            let old_pos = state.players[state.current_player_index].position;
            let new_pos = if old_pos >= spaces {
                old_pos - spaces
            } else {
                40 - (spaces - old_pos)
            };
            state.players[state.current_player_index].position = new_pos;
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::GoToJail => {
            monopoly_core::send_to_jail(state);
        }

        CardEffect::GetOutOfJail => {
            state.players[state.current_player_index].get_out_of_jail += 1;
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::CollectFromPlayers { amount } => {
            let current_id = state.players[state.current_player_index].id;
            let active: Vec<u8> = state.players.iter()
                .filter(|p| !p.bankrupt && p.id != current_id)
                .map(|p| p.id)
                .collect();

            for pid in active {
                if let Some(payer) = state.players.iter_mut().find(|p| p.id == pid) {
                    payer.money = payer.money.saturating_sub(amount);
                }
                if let Some(collector) = state.players.iter_mut()
                    .find(|p| p.id == current_id) {
                    collector.money += amount;
                }
            }
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::PayPerPlayer { amount } => {
            let current_id = state.players[state.current_player_index].id;
            let active: Vec<u8> = state.players.iter()
                .filter(|p| !p.bankrupt && p.id != current_id)
                .map(|p| p.id)
                .collect();

            let total = amount * active.len() as u32;
            state.players[state.current_player_index].money =
                state.players[state.current_player_index].money.saturating_sub(total);

            for pid in active {
                if let Some(p) = state.players.iter_mut().find(|p| p.id == pid) {
                    p.money += amount;
                }
            }
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::PayPerHouse { per_house, per_hotel } => {
            let current_id = state.players[state.current_player_index].id;
            let player = state.players.iter().find(|p| p.id == current_id).unwrap();
            let properties = player.properties.clone();

            let mut total = 0u32;
            for tile_index in properties {
                if let Tile::Property(p) = &state.board.tiles[tile_index] {
                    if p.houses < 5 {
                        total += per_house * p.houses as u32;
                    } else {
                        total += per_hotel;
                    }
                }
            }

            state.players[state.current_player_index].money =
                state.players[state.current_player_index].money.saturating_sub(total);
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::MoveToNearestRailroad => {
            let pos = state.players[state.current_player_index].position;
            // Railroad positions: 5, 15, 25, 35
            let railroads = [5usize, 15, 25, 35];
            let nearest = railroads.iter()
                .min_by_key(|&&r| {
                    if r >= pos { r - pos } else { 40 - pos + r }
                })
                .copied()
                .unwrap_or(5);

            let old_pos = pos;
            state.players[state.current_player_index].position = nearest;
            if nearest < old_pos {
                state.players[state.current_player_index].money += go_salary;
            }
            state.turn_phase = TurnPhase::PostRoll;
        }

        CardEffect::MoveToNearestUtility => {
            let pos = state.players[state.current_player_index].position;
            // Utility positions: 12 (Electric), 28 (Water Works)
            let utilities = [12usize, 28];
            let nearest = utilities.iter()
                .min_by_key(|&&u| {
                    if u >= pos { u - pos } else { 40 - pos + u }
                })
                .copied()
                .unwrap_or(12);

            let old_pos = pos;
            state.players[state.current_player_index].position = nearest;
            if nearest < old_pos {
                state.players[state.current_player_index].money += go_salary;
            }
            state.turn_phase = TurnPhase::PostRoll;
        }
    }
}
