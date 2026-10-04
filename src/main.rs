use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, Paragraph},
};
use serde::Deserialize;
use std::time::Duration;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

mod dnd_repository;
mod repository_error;

use dnd_repository::DnDRepository;

#[derive(Debug, Deserialize)]
pub struct MonsterEntry {
    index: String,
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct MonsterList {
    count: usize,
    results: Vec<MonsterEntry>,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct Model {
    /// Is the application running?
    running: bool,
    /// The current content of the search bar.
    query: String,
    /// Cursor position in the search bar, as a character index into `query`.
    cursor: usize,
    /// The monsters matchings the current query.
    results: Vec<String>,
    /// Selection and scroll state of the result list.
    list_state: ListState,

    search: Input,

    all_monsters: Vec<MonsterEntry>,
}

#[derive(PartialEq, Debug)]
enum Message {
    MoveListSelection(ListDirection),
    MoveSearchCursor(Event),
    UpdateSearchQuery(Event),
    Quit,
}

#[derive(PartialEq, Debug)]
enum ListDirection {
    Up,
    Down,
}

impl Model {
    /// Construct a new instance of [`Model`].
    pub fn new() -> Self {
        let mut app = Self::default();
        let monster_repo =
            DnDRepository::<MonsterList>::new("http://localhost:3000/api/2014/monsters");
        app.all_monsters = monster_repo.get().unwrap().results;
        app.update_results();
        app
    }

    /// Recomputes `results` from the current query and resets the selection.
    fn update_results(&mut self) {
        let query = self.search.value().to_lowercase();
        self.results = self
            .all_monsters
            .iter()
            .map(|me| &me.name)
            .filter(|name| name.to_lowercase().contains(&query))
            .cloned()
            .collect();
        self.list_state
            .select((!self.results.is_empty()).then_some(0));
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let mut model = Model::new();
    model.running = true;
    while model.running {
        let mut current_msg = handle_key()?;
        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }

        terminal.draw(|frame| view(&mut model, frame))?;
    }
    let result = Ok(());
    ratatui::restore();
    result
}

/// Reads the crossterm events and passes back a corresponding message.
fn handle_key() -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        let event = event::read()?;
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            return Ok(match (key.modifiers, key.code) {
                (_, KeyCode::Esc)
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                    Some(Message::Quit)
                }
                (_, KeyCode::Down) => Some(Message::MoveListSelection(ListDirection::Down)),
                (_, KeyCode::Up) => Some(Message::MoveListSelection(ListDirection::Up)),
                (_, KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End) => {
                    Some(Message::MoveSearchCursor(event))
                }
                (
                    KeyModifiers::NONE | KeyModifiers::SHIFT,
                    KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete,
                ) => Some(Message::UpdateSearchQuery(event)),
                _ => None,
            });
        }
    }
    Ok(None)
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::MoveSearchCursor(event) => {
            model.search.handle_event(&event);
        }
        Message::UpdateSearchQuery(event) => {
            model.search.handle_event(&event);
            model.update_results();
        }
        Message::MoveListSelection(direction) => match direction {
            ListDirection::Up => model.list_state.select_previous(),
            ListDirection::Down => model.list_state.select_next(),
        },
        Message::Quit => model.running = false,
    }
    None
}

/// Renders the user interface.
fn view(model: &mut Model, frame: &mut Frame) {
    let width = frame.area().width.max(3) - 3;
    let scroll = model.search.visual_scroll(width as usize);

    let [search_area, results_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = Line::from("Monster Search").bold().blue().left_aligned();
    let search = Paragraph::new(model.search.value()).block(Block::bordered().title(title));
    frame.render_widget(search, search_area);

    let x = model.search.visual_cursor().max(scroll) - scroll + 1;
    frame.set_cursor_position((search_area.x + x as u16, search_area.y + 1));

    let results_title = format!("Results ({})", model.results.len());
    let results = List::new(model.results.iter().cloned())
        .block(Block::bordered().title(results_title))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    frame.render_stateful_widget(results, results_area, &mut model.list_state);

    let help = Line::from("Type to search | ↑/↓ select | Esc or Ctrl-C quit").dim();
    frame.render_widget(help, help_area);
}
