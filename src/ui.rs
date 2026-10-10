use crate::app::bestiary;
use crate::model::Model;
use ratatui::Frame;

/// Renders the user interface.
pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    bestiary::view(&mut model.bestiary_model, frame, frame.area());
}
