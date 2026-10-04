mod app;
mod dnd_api_client;
mod monster;
mod ui;

use crate::app::App;
use crate::dnd_api_client::ClientError;
use crate::monster::Monster;
use ratatui::widgets::ListState;
use tui_input::{Input, InputRequest};

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
struct Model {
    /// Is the application running?
    running: bool,
    /// The search bar.
    search: Input,
    /// Every monster known to the API, loaded once at startup.
    all_monsters: Vec<Monster>,
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
    fn new() -> Result<Self, ClientError> {
        let mut model = Self {
            all_monsters: dnd_api_client::all_monsters()?,
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
    let app = App::new()?;
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
