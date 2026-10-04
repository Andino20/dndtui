# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

dndtui is a Ratatui terminal app for Dungeon Masters. The goal is monster lookup (stat blocks fetched from the D&D 5e API) and combat encounter management (initiative, hit points, turns). See `README.md` for the idea.

The codebase is at a very early stage and the author considers the existing code provisional. Only a monster name search exists. Do not treat current structure as a convention to preserve; ask before large restructurings, but do not copy patterns just because they are there.

## Commands

```sh
cargo run                    # needs the D&D 5e API on localhost:3000, see below
cargo build
cargo fmt -- --check         # CI check
cargo clippy                 # CI check
cargo test --locked --all-features --all-targets   # CI check
cargo test <name>            # run a single test by name filter
cargo doc --no-deps --all-features                 # CI check, runs on nightly there
```

There are no tests yet.

## Runtime dependency

`Model::new` in `src/main.rs` fetches the monster list from `http://localhost:3000/api/2014/monsters` and unwraps the result, so the app panics on startup unless a local instance of the [5e-srd-api](https://github.com/5e-bits/5e-srd-api) is running. The URL is hardcoded. Since the app takes over the terminal, it cannot be meaningfully exercised from a non-interactive shell; verify changes with `cargo build`, `cargo clippy` and tests.

## Architecture

`src/main.rs` follows The Elm Architecture as used in the Ratatui docs:

- `Model` holds all application state.
- `handle_key` turns crossterm key events into a `Message`.
- `update(&mut Model, Message) -> Option<Message>` is the only place state changes in response to input. It may return a follow-up message, which the main loop feeds back into `update` until it returns `None`.
- `view` renders the model into a frame. It takes `&mut Model` only because Ratatui's stateful list widget needs mutable `ListState`.

New behavior should be added as a `Message` variant handled in `update`, not as ad-hoc mutation in `handle_key` or `view`.

Data access:

- `DnDRepository<T>` (`src/dnd_repository.rs`) is a generic blocking HTTP getter that deserializes the response of one URL into `T`. Errors are `RepositoryError` (`thiserror`).
- Requests use `reqwest::blocking` on the UI thread, so a request freezes the interface for its duration (5 s timeout).

The full monster list is loaded once at startup and the search filters it in memory by case-insensitive substring; the API is not queried per keystroke.

The search cursor is tracked as a character index, not a byte index; use `cursor_byte_index` when slicing or editing `query`.
