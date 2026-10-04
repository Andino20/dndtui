# dndtui

A terminal application for Dungeon Masters: look up monsters and run combat encounters without leaving the terminal.

The project is at a very early stage. So far only a monster name search exists; everything else described under [Idea](#idea) is the plan, not the current state.

## Current state

- The list of monster names is loaded from the API once at startup.
- Typing filters the list by case-insensitive substring match.
- The result list can be navigated with the arrow keys.

| Key | Action |
|---|---|
| any text | edit the search query |
| `↑` / `↓` | move the selection in the result list |
| `Esc` or `Ctrl-C` | quit |

Stat blocks and encounter management are not implemented yet.

## Idea

Running a D&D session means constantly switching between rulebooks, notes and trackers. dndtui aims to put the two things a DM needs most during combat into one keyboard-driven interface:

- **Monster lookup**: search monsters by name and view their stat blocks, such as armor class, hit points, ability scores, actions and special abilities.
- **Encounter management**: build combat encounters from monsters and player characters, then track initiative order, hit points and whose turn it is while the fight is running.

Monster data is not bundled with the app. It is fetched from the [D&D 5e API], which serves the SRD content.

## Running

You need a recent Rust toolchain.

```sh
cargo run
```

At the moment the app expects an instance of the D&D 5e API at `http://localhost:3000`. The API's [repository][5e-srd-api] describes how to run one locally.

## Caveats

- **A local API is required.** The address `http://localhost:3000` is hardcoded and cannot be configured.
- **The app does not start without the API.** If the monster list cannot be loaded, the app exits with an error report instead of opening the interface.
- **Startup can take a few seconds.** The monster list is fetched before the interface appears, with a timeout of 5 seconds.
- **Only the 2014 rules are used.** The monster list comes from the API's 2014 SRD endpoint.
- **Selecting a monster does nothing yet.** The search shows names only.
- **The code is provisional.** Structure and behaviour may change substantially.

## Built with

- [Ratatui], [crossterm] and [tui-input] for the terminal UI
- [reqwest] and [serde] for talking to the API
- [thiserror] and [color-eyre] for error handling

## License

Copyright (c) Andreas Schlager <andreas.schlager@stud.plus.ac.at>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[D&D 5e API]: https://www.dnd5eapi.co
[5e-srd-api]: https://github.com/5e-bits/5e-srd-api
[Ratatui]: https://ratatui.rs
[crossterm]: https://github.com/crossterm-rs/crossterm
[reqwest]: https://github.com/seanmonstar/reqwest
[serde]: https://serde.rs
[tui-input]: https://github.com/sayanarijit/tui-input
[thiserror]: https://github.com/dtolnay/thiserror
[color-eyre]: https://github.com/eyre-rs/eyre/tree/master/color-eyre
[LICENSE]: ./LICENSE
