# dndtui

A terminal application for Dungeon Masters: look up monsters and run combat encounters without leaving the terminal.

The project is at a very early stage. So far only a basic monster name search exists; everything else described here is the plan, not the current state.

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

## Built with

- [Ratatui] and [crossterm] for the terminal UI
- [reqwest] and [serde] for talking to the API

## License

Copyright (c) Andreas Schlager <andreas.schlager@stud.plus.ac.at>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[D&D 5e API]: https://www.dnd5eapi.co
[5e-srd-api]: https://github.com/5e-bits/5e-srd-api
[Ratatui]: https://ratatui.rs
[crossterm]: https://github.com/crossterm-rs/crossterm
[reqwest]: https://github.com/seanmonstar/reqwest
[serde]: https://serde.rs
[LICENSE]: ./LICENSE
