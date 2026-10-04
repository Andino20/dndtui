use ratatui::widgets::ListState;
use serde::Deserialize;
use tui_input::{Input, InputRequest};

mod app;
mod dnd_repository;
mod repository_error;
mod ui;
mod dnd_api_client;
mod monster;

use crate::app::App;
use dnd_repository::DnDRepository;
use repository_error::RepositoryError;

const MONSTERS_URL: &str = "http://localhost:3000/api/2014/monsters";

#[derive(Debug, Deserialize)]
struct MonsterEntry {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MonsterList {
    results: Vec<MonsterEntry>,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
struct Model {
    /// Is the application running?
    running: bool,
    /// The search bar.
    search: Input,
    /// Every monster known to the API, loaded once at startup.
    all_monsters: Vec<MonsterEntry>,
    /// Indices into `all_monsters` of the monsters matching the current query.
    results: Vec<usize>,
    /// Selection and scroll state of the result list.
    list_state: ListState,
}

#[derive(PartialEq, Debug)]
enum Message {
    MoveListSelection(ListDirection),
    EditSearch(InputRequest),
    Quit,
}

#[derive(PartialEq, Debug)]
enum ListDirection {
    Up,
    Down,
}

impl Model {
    /// Construct a new instance of [`Model`], loading the monster list from the API.
    fn new() -> Result<Self, RepositoryError> {
        let mut model = Self {
            all_monsters: DnDRepository::<MonsterList>::new(MONSTERS_URL)
                .get()?
                .results,
            ..Self::default()
        };
        model.update_results();
        Ok(model)
    }

    /// Recomputes `results` from the current query and resets the selection.
    fn update_results(&mut self) {
        let query = self.search.value().to_lowercase();
        self.results = self
            .all_monsters
            .iter()
            .enumerate()
            .filter(|(_, monster)| monster.name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
        self.list_state
            .select((!self.results.is_empty()).then_some(0));
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let app = App::new();
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
