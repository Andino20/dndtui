use crate::Model;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::{Line, Modifier, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, List, Paragraph};

/// Renders the user interface.
pub(crate) fn view(model: &mut Model, frame: &mut Frame) {
    let [search_area, results_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    // Two columns for the border, one to keep the cursor visible at the end of the query.
    let width = search_area.width.saturating_sub(3);
    let scroll = model.search.visual_scroll(width as usize);

    let title = Line::from("Monster Search").bold().blue().left_aligned();
    let search = Paragraph::new(model.search.value())
        .scroll((0, scroll as u16))
        .block(Block::bordered().title(title));
    frame.render_widget(search, search_area);

    let cursor = model.search.visual_cursor().saturating_sub(scroll) + 1;
    frame.set_cursor_position((search_area.x + cursor as u16, search_area.y + 1));

    let results_title = format!("Results ({})", model.results.len());
    let names = model
        .results
        .iter()
        .map(|&index| model.all_monsters[index].name.as_str());
    let results = List::new(names)
        .block(Block::bordered().title(results_title))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    frame.render_stateful_widget(results, results_area, &mut model.list_state);

    let help = Line::from("Type to search | ↑/↓ select | Esc or Ctrl-C quit").dim();
    frame.render_widget(help, help_area);
}
