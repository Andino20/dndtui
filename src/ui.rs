use crate::app::bestiary;
use crate::model::{Model, Page};
use ratatui::layout::{Layout, Rect};
use ratatui::prelude::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::Tabs;
use ratatui::{Frame, symbols};

/// Renders the user interface.
pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    let [tab_area, content_area] =
        Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    render_tabs(&model.active_page, frame, tab_area);
    match model.active_page {
        Page::Bestiary => bestiary::view(&mut model.bestiary_model, frame, content_area),
        Page::Combat => {}
    }
}

fn render_tabs(active_page: &Page, frame: &mut Frame, area: Rect) {
    let selected_tab = match active_page {
        Page::Bestiary => 0,
        Page::Combat => 1,
    };
    let tabs = Tabs::new(vec!["Bestiary", "Combat"])
        .style(Color::White)
        .highlight_style(Style::default().magenta().on_black().bold())
        .select(selected_tab)
        .divider(symbols::DOT)
        .padding(" ", " ");
    frame.render_widget(tabs, area);
}
