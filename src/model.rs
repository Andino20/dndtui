use crate::app::Result;
use crate::app::bestiary::BestiaryModel;
use crate::app::combat::CombatModel;
use crate::dnd_api_client::DndApiClient;

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct Model {
    /// Is the application running?
    pub running: bool,

    pub api_client: DndApiClient,

    pub bestiary_model: BestiaryModel,

    pub combat_model: CombatModel,
}

impl Model {
    /// Construct a new instance of [`crate::Model`], loading the monster list from the API.
    pub fn new() -> Result<Self> {
        let api_client = DndApiClient::new("http://localhost:3000/api/2014")?;
        let bestiary_model = BestiaryModel::new(&api_client)?;
        let model = Self {
            api_client,
            bestiary_model,
            ..Self::default()
        };
        Ok(model)
    }
}
