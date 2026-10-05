mod app;
mod dnd_api_client;
mod model;
mod monster;
mod ui;

use crate::app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let app = App::new()?;
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
