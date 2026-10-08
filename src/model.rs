use crate::dnd_api_client::{ClientError, DndApiClient};
use crate::monster::{Monster, MonsterEntry};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use tui_input::Input;

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct Model {
    /// Is the application running?
    pub running: bool,
    /// The search bar.
    pub search: Input,
    /// Every monster known to the API, loaded once at startup.
    pub all_monsters: Vec<MonsterEntry>,
    /// Cached monster details requested from the API during runtime.
    pub monsters: HashMap<String, Monster>,
    /// Indices into `all_monsters` of the monsters matching the current query.
    pub results: Vec<usize>,
    /// Selection and scroll state of the result list.
    pub list_state: ListState,

    pub detailed_monster_id: Option<String>,

    pub api_client: DndApiClient,
}

impl Model {
    /// Construct a new instance of [`crate::Model`], loading the monster list from the API.
    pub fn new() -> Result<Self, ClientError> {
        let api_client = DndApiClient::new("http://localhost:3000/api/2014")?;
        let mut model = Self {
            api_client,
            ..Self::default()
        };
        model.all_monsters = model.api_client.all_monsters()?;
        model.update_results();
        Ok(model)
    }

    /// Recomputes `results` from the current query and resets the selection.
    pub fn update_results(&mut self) {
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
