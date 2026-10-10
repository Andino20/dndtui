# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

dndtui is a Ratatui terminal app for Dungeon Masters. The goal is monster lookup (stat blocks fetched from the D&D 5e
API) and combat encounter management (initiative, hit points, turns). See `README.md` for the idea.

The codebase is at a very early stage and the author considers the existing code provisional. The bestiary page (monster
name search and a details pane) exists; the combat page is an empty stub that is not wired up yet. Do not treat current
structure as a convention to preserve; ask before large restructurings, but do not copy patterns just because they are
there.

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

`App::new` in `src/app.rs` calls `Model::new` (`src/model.rs`), which creates the `DndApiClient` and passes it to
`BestiaryModel::new`, which fetches the monster list from `http://localhost:3000/api/2014/monsters`. This happens in
`main` before `ratatui::run` takes over the terminal. The base URL is passed to `DndApiClient::new` and is hardcoded in
`Model::new`.

If the request fails, `App::new` returns the error and `main` exits with a color-eyre report before the interface opens.
This is deliberate for now, since the app is useless without the monster list; once it can do something without the API,
the error should be shown in the interface instead. To run the app, a local instance of
the [5e-srd-api](https://github.com/5e-bits/5e-srd-api) must be running.

Since the app takes over the terminal, it cannot be meaningfully exercised from a non-interactive shell; verify changes
with `cargo build`, `cargo clippy` and tests.

## Architecture

The app follows The Elm Architecture as used in the Ratatui docs, with one sub-module per page. Each page owns its
model, its message type and its `update`, `handle_key` and `view` functions; `App` only routes between them.

- `src/main.rs`: module declarations and `main`.
- `src/model.rs`: `Model`, the top-level state: `running`, the `DndApiClient` and one model per page.
- `src/app.rs`: `App`, which owns the `Model`, the private top-level `Message` enum, and the error type `DnDTuiError`.
- `src/app/bestiary.rs`: the bestiary page: `BestiaryModel`, `BestiaryMessage`, `ListDirection`, and the free functions
  `update`, `handle_key` and `view`, plus the stat block rendering helpers.
- `src/app/combat.rs`: the combat page. Only an empty `CombatModel` and an empty `view` so far; nothing calls it.
- `src/ui.rs`: the top-level `view`, which currently just delegates to `bestiary::view`.
- `src/dnd_api_client.rs`: the API client and its error type `ClientError`, see Data access.
- `src/monster.rs`: the domain types `MonsterEntry` (list entry) and `Monster` (full stat block) with its parts.

The loop in `App::run`:

- `terminal.draw` calls `ui::view`. It takes `&mut Model` only because Ratatui's stateful list widget needs mutable
  `ListState`.
- `App::handle_key` waits up to 250 ms for a crossterm event, drops everything but key presses, handles the global
  shortcut (Ctrl-C) and passes any other `KeyEvent` to the page's `handle_key`, wrapping the result in
  `Message::Bestiary`.
- `App::update(&mut Model, Message) -> Result<Option<Message>>` is the only place state changes in response to input.
  It may return a follow-up message, which `run` feeds back into `update` until it returns `None`.

`handle_key` and `update` are associated functions without `self`. `Model::running` is set to `true` only at the start
of `App::run` and cleared by `Message::Quit` (Ctrl-C) or `BestiaryMessage::Leave`.

Pages and their parent:

- A page only knows its own types. `bestiary.rs` imports nothing from `app`; `bestiary::update` and
  `bestiary::handle_key` return `Option<BestiaryMessage>`, and `App` wraps them with `Message::Bestiary`.
- A page's `update` cannot fail and has no access to the API client. When it needs something from the parent it returns
  a message that `App::update` intercepts before delegating. Today that is `BestiaryMessage::RequestMonsterById`: `App`
  fetches the monster, stores it in `BestiaryModel::monsters` and follows up with `OpenMonsterDetails`. The page's own
  `update` has an explicit no-op arm for it. `BestiaryMessage::Leave` works the same way: Esc maps to
  `BestiaryMessage::Back`, which closes the details pane or, if nothing is open, returns `Leave`, which `App` currently
  treats as quit.
- Matches over message enums are exhaustive without a `_` arm, so a new variant is a compile error until handled.
- Message variants are written qualified (`Message::Quit`, `BestiaryMessage::EditSearch`), not imported bare.

New behavior should be added as a message variant handled in the page's `update` (or in `App::update` if it is global
or needs shared state), not as ad-hoc mutation in `handle_key` or `view`.

Known gaps in the page structure, not conventions:

- There is no notion of an active page; `ui::view` and `App::handle_key` are hardwired to the bestiary.
- The monster cache lives in `BestiaryModel`, although combat will likely need it too.

Error handling:

- Modules define typed errors with `thiserror`: `ClientError` (`src/dnd_api_client.rs`) and `DnDTuiError`
  (`src/app.rs`). Each of the two modules has a private `Result<T>` alias for its own error type.
- A function returns the error type of its own layer. `Model::new` and `BestiaryModel::new` return `ClientError`;
  `App::new` and `App::update` convert it into `DnDTuiError::ClientError` through `#[from]`, which keeps the underlying
  error as the source.
- `main` returns `color_eyre::Result`, and typed errors convert into the report through `?`. A `Result` with a typed
  error returned as a tail expression does not convert; it needs `?` followed by `Ok(())`.
- `DnDTuiError::TerminalError` wraps `std::io::Error`, which is the error type of the crossterm backend behind
  `DefaultTerminal`. It is produced by `terminal.draw` and also by `event::poll` and `event::read`, although its message
  only mentions drawing.
- Naming the source as `<CrosstermBackend<Stdout> as Backend>::Error` in a `#[from]` attribute does not compile (E0119),
  so use the concrete `std::io::Error`.
- An error from `App::update` ends the main loop, so a failed monster details request currently exits the app.

Data access:

- `src/dnd_api_client.rs` is the only module that talks to the API. `DndApiClient` holds the `reqwest` client and the
  base URL and exposes one public method per query: `all_monsters() -> Result<Vec<MonsterEntry>>` and
  `monster_by_id(id) -> Result<Monster>`.
- The JSON bodies are deserialized into private `*Response` structs that mirror the API, then mapped to domain types
  through `From` impls. Response structs do not leave the module; the rest of the app only sees domain types.
- The private `get<T: DeserializeOwned>(url)` does the HTTP request and deserialization for all queries. It takes the
  path relative to the base URL.
- Every request carries an `x-forwarded-proto: https` header, because the API redirects plain HTTP to HTTPS in
  production mode unless a proxy vouches for the request.
- Requests use `reqwest::blocking` with a 5 s timeout. The monster list is requested at startup. Monster details are
  requested from the main loop when an uncached monster is opened, which freezes the interface for the duration of the
  request.

The full monster list is loaded once at startup and the search filters it in memory by case-insensitive substring; the
API is not queried per keystroke. Monster details are fetched on first open and cached by id in
`BestiaryModel::monsters`.

The search bar is a `tui_input::Input`. Key events that are not shortcuts are converted with `to_input_request` and
applied in `bestiary::update` via `BestiaryMessage::EditSearch`; the result list is only recomputed when the input
reports that its value changed. `BestiaryModel::results` holds indices into `BestiaryModel::all_monsters`, not copies of
the names.
