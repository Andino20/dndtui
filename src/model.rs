use crate::app::bestiary::BestiaryModel;
use crate::app::combat::CombatModel;
use crate::dnd_api_client::{ClientError, DndApiClient};

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct Model {
    /// Is the application running?
    pub running: bool,

    pub api_client: DndApiClient,

    pub active_page: Page,

    pub bestiary_model: BestiaryModel,

    pub combat_model: CombatModel,
}

impl Model {
    /// Construct a new instance of [`Model`], loading the monster list from the API.
    pub fn new() -> Result<Self, ClientError> {
        let api_client = DndApiClient::new("http://localhost:3000/api/2014")?;
        let bestiary_model = BestiaryModel::new(&api_client)?;
        Ok(Self {
            running: false,
            api_client,
            active_page: Page::Bestiary,
            bestiary_model,
            combat_model: CombatModel::default(),
        })
    }
}

#[derive(Debug)]
pub enum Page {
    Bestiary,
    Combat,
}

impl Page {
    pub fn next_page(&self) -> Page {
        match self {
            Page::Bestiary => Page::Combat,
            Page::Combat => Page::Bestiary,
        }
    }
}
