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

`App::new` in `src/app.rs` calls `Model::new`, which fetches the monster list from `http://localhost:3000/api/2014/monsters`. This happens in `main` before `ratatui::run` takes over the terminal. The URL is hardcoded as `MONSTERS_URL` in `src/main.rs`.

If the request fails, `App::new` discards the `RepositoryError` and falls back to `Model::default()`, so the app starts with an empty monster list and no error message. The author knows about this and accepts it for now. To see monsters, a local instance of the [5e-srd-api](https://github.com/5e-bits/5e-srd-api) must be running.

Since the app takes over the terminal, it cannot be meaningfully exercised from a non-interactive shell; verify changes with `cargo build`, `cargo clippy` and tests.

## Architecture

The app follows The Elm Architecture as used in the Ratatui docs, split over these files:

- `src/main.rs`: `Model` (all application state), `Message`, `ListDirection`, the API response types `MonsterEntry` and `MonsterList`, and `main`.
- `src/app.rs`: `App`, which owns the `Model`, and the error type `DnDTuiError`.
- `src/ui.rs`: `view`, which renders the model into a frame.

The loop in `App::run`:

- `terminal.draw` calls `ui::view`. It takes `&mut Model` only because Ratatui's stateful list widget needs mutable `ListState`.
- `App::handle_key` waits up to 250 ms for a crossterm event and turns a key press into a `Message`.
- `App::update(&mut Model, Message) -> Option<Message>` is the only place state changes in response to input. It may return a follow-up message, which `run` feeds back into `update` until it returns `None`.

`handle_key` and `update` are associated functions without `self`. `Model::running` is set to `true` only at the start of `App::run` and cleared by `Message::Quit`.

New behavior should be added as a `Message` variant handled in `update`, not as ad-hoc mutation in `handle_key` or `view`.

Error handling:

- Modules define typed errors with `thiserror`: `RepositoryError` (`src/repository_error.rs`) and `DnDTuiError` (`src/app.rs`).
- `main` returns `color_eyre::Result`, and typed errors convert into the report through `?`. A `Result` with a typed error returned as a tail expression does not convert; it needs `?` followed by `Ok(())`.
- `DnDTuiError::TerminalError` wraps `std::io::Error`, which is the error type of the crossterm backend behind `DefaultTerminal`. It is produced by `terminal.draw` and also by `event::poll` and `event::read`, although its message only mentions drawing.
- Naming the source as `<CrosstermBackend<Stdout> as Backend>::Error` in a `#[from]` attribute does not compile (E0119), so use the concrete `std::io::Error`.

Data access:

- `DnDRepository<T>` (`src/dnd_repository.rs`) is a generic blocking HTTP getter that deserializes the response of one URL into `T`.
- Requests use `reqwest::blocking` with a 5 s timeout. The only request today is the one at startup; a request made from the main loop would freeze the interface for its duration.

The full monster list is loaded once at startup and the search filters it in memory by case-insensitive substring; the API is not queried per keystroke.

The search bar is a `tui_input::Input`. Key events that are not app shortcuts are converted with `to_input_request` and applied in `update` via `Message::EditSearch`; the result list is only recomputed when the input reports that its value changed. `Model::results` holds indices into `Model::all_monsters`, not copies of the names.
