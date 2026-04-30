# Monopoly

A serious, full-featured Monopoly implementation built in Rust — playable over the network with friends, and designed from the ground up to be extensible with custom game modes and special mechanics.

This isn't a toy clone. The goal is a clean, maintainable codebase that plays a complete game of Monopoly correctly, with a architecture that makes adding new rules, mechanics, and modes straightforward rather than a hack.

## Features (in progress)

- Full standard Monopoly rules — auctions, trading, mortgages, houses, hotels, all cards
- Online multiplayer over TCP — host a game, invite friends by IP
- Modular ruleset system — swap in custom modes without touching core logic
- Data-driven board and cards — defined in TOML config, not hardcoded
- Clean GUI via raylib

## Architecture

crates/
├── monopoly-core/     Pure game logic — no I/O, fully testable in isolation
├── monopoly-server/   TCP server, lobby, authoritative game state
└── monopoly-client/ raylib GUI client

The core crate has zero networking and zero rendering dependencies. Game state lives on the server — clients send actions and receive state snapshots. This keeps the logic clean and makes the game easy to test without spinning up a server.

## Stack

| Layer | Crate |
|---|---|
| Rendering | `raylib` |
| Networking | `tokio` + TCP |
| Serialization | `serde` + `bincode` |
| Config / Rulesets | `toml` + `serde` |

## Running

```bash
# Start the server (host machine)
cargo run -p monopoly-server

# Start the client
cargo run -p monopoly-client
```

## Roadmap

- [x] Workspace scaffold
- [X] Core data model
- [X] RuleSet system
- [X] Board definition (40 tiles, TOML config) 
- [X] Dice, movement, rent, buying, jail logic
- [ ] Buildings and mortgages
- [ ] Trading 
- [ ] Bankruptcy and win condition
- [ ] Networking
- [ ] GUI
- [ ] Custom game modes



