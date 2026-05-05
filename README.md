# Monopoly

A serious, multiplayer Monopoly implementation in Rust — playable over the network 
with friends, designed from the ground up to be extensible with custom game modes 
and special mechanics.

This isn't a toy clone. The goal is a clean, maintainable codebase that plays a 
complete game of Monopoly correctly, with an architecture that makes adding new 
rules, modes, and mechanics straightforward rather than a hack.

## Status

`v0.4.0` — Interactions complete. Property cards, trading, mortgage, build all wired up.
Jail edge cases and tooltip in trade screen coming in v0.5.0.

## Features

- Full standard Monopoly rules — auctions, rent, jail, bankruptcy
- Online multiplayer over TCP — host a game, invite friends by IP
- Modular ruleset system — swap in custom modes without touching core logic
- Data-driven board and cards — defined in TOML config, not hardcoded
- macroquad GUI — board rendering, player tokens, action panel
- Theme system coming in v0.3.0

## Architecture

crates/
├── monopoly-core/     Pure game logic — no I/O, fully testable in isolation
├── monopoly-server/   TCP server, lobby, authoritative game state
└── monopoly-client/   macroquad GUI client

The core crate has zero networking and zero rendering dependencies.
Game state lives on the server — clients send actions and receive state snapshots.

## Stack

| Layer | Crate |
|---|---|
| Rendering | `macroquad` |
| Networking | `tokio` + TCP |
| Serialization | `serde` + `bincode` |
| Config / Rulesets | `toml` + `serde` |

## Running

```bash
# Start the server (host machine)
cargo run -p monopoly-server

# Start the client (connect to host IP)
cargo run -p monopoly-client
```

For internet play, the host needs to forward port `7777` on their router.

## Roadmap

### v0.1.0 — Foundation ✓
- [x] Workspace scaffold
- [x] Core data model
- [x] RuleSet system
- [x] Board definition (40 tiles, TOML config)

### v0.2.0 — Gameplay + Networking ✓
- [x] Dice, movement, rent, buying, jail logic
- [x] Buildings and mortgages
- [x] Trading
- [x] Bankruptcy and win condition
- [x] Packet design (ClientMessage / ServerMessage)
- [x] Server lobby, session handling, game engine
- [x] Client connection layer
- [x] macroquad GUI — board, players, action panel
- [x] Auction system (simultaneous bidding)

### v0.3.0 — UI Polish ✓
- [x] Theme system (Classic, Midnight, Retro)
- [x] Dashboard game screen with card panels
- [x] Board tile hover tooltips with full property details
- [x] Property card collection panel with horizontal scroll
- [x] Community Chest and Chance card decks (standard set)
- [x] Data-driven card decks via TOML config
- [x] Player color consistency across board and panels
- [x] Connect screen and lobby screen themed
- [x] Soft ambient background, no harsh whites

### v0.4.0 — Interactions ✓
- [x] Property card detail view on click
- [x] Mortgage and build actions from card view
- [x] Trading screen with property selection and money input
- [x] Card panel click offset fix
- [x] Trade screen card overflow with per-column scroll
- [x] Tooltip blocked during trade and detail screens
- [x] game.rs refactored into 6 focused modules

### v0.5.0 — Polish and fixes (next)
- [ ] Jail edge cases — doubles tracking, consecutive doubles
- [ ] Tooltip on trade screen property cards
- [ ] Trade accept/reject proper UI prompt
- [ ] Sell house from card detail view
- [ ] Building validation feedback to client

### v0.6.0 — Extensibility
- [ ] Custom ruleset hookpoints
- [ ] Mode selection in lobby
- [ ] Relay server / room codes (v2 networking)

### v1.0.0 — Release
- [ ] Full rule correctness pass
- [ ] Reconnection handling
- [ ] Sound effects
- [ ] Packaging and distribution
