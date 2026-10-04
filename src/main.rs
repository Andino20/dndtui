use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, Paragraph},
};
use std::time::Duration;
use serde::Deserialize;
use tui_input::Input;

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
    MoveSearchCursor(CursorPosition),
    MoveListSelection(ListDirection),
    UpdateSearchQuery(QueryUpdate),
    Quit,
}

#[derive(PartialEq, Debug)]
enum CursorPosition {
    Left,
    Right,
    Home,
    End,
}

#[derive(PartialEq, Debug)]
enum ListDirection {
    Up,
    Down,
}

#[derive(PartialEq, Debug)]
enum QueryUpdate {
    Insert(char),
    Delete,
    Backspace,
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

    /// Byte offset into `query` corresponding to the character-based `cursor`.
    fn cursor_byte_index(&self) -> usize {
        self.query
            .char_indices()
            .nth(self.cursor)
            .map_or(self.query.len(), |(i, _)| i)
    }

    fn insert_char(&mut self, c: char) {
        let index = self.cursor_byte_index();
        self.query.insert(index, c);
        self.cursor += 1;
        self.update_results();
    }

    fn delete_char_before_cursor(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.delete_char_at_cursor();
    }

    fn delete_char_at_cursor(&mut self) {
        let index = self.cursor_byte_index();
        if index < self.query.len() {
            self.query.remove(index);
            self.update_results();
        }
    }

    /// Recomputes `results` from the current query and resets the selection.
    fn update_results(&mut self) {
        let query = self.query.to_lowercase();
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
        };

        terminal.draw(|frame| view(&mut model, frame))?;
    }
    let result = Ok(());
    ratatui::restore();
    result
}

/// Reads the crossterm events and passes back a corresponding message.
fn handle_key() -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        return Ok(match (key.modifiers, key.code) {
            (_, KeyCode::Esc)
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                Some(Message::Quit)
            }
            (_, KeyCode::Down) => Some(Message::MoveListSelection(ListDirection::Down)),
            (_, KeyCode::Up) => Some(Message::MoveListSelection(ListDirection::Up)),
            (_, KeyCode::Left) => Some(Message::MoveSearchCursor(CursorPosition::Left)),
            (_, KeyCode::Right) => Some(Message::MoveSearchCursor(CursorPosition::Right)),
            (_, KeyCode::Home) => Some(Message::MoveSearchCursor(CursorPosition::Home)),
            (_, KeyCode::End) => Some(Message::MoveSearchCursor(CursorPosition::End)),
            (_, KeyCode::Backspace) => Some(Message::UpdateSearchQuery(QueryUpdate::Backspace)),
            (_, KeyCode::Delete) => Some(Message::UpdateSearchQuery(QueryUpdate::Delete)),
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                Some(Message::UpdateSearchQuery(QueryUpdate::Insert(c)))
            }
            _ => None,
        });
    }
    Ok(None)
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::MoveSearchCursor(position) => match position {
            CursorPosition::Left => model.cursor = model.cursor.saturating_sub(1),
            CursorPosition::Right => {
                model.cursor = (model.cursor + 1).min(model.query.chars().count())
            }
            CursorPosition::Home => model.cursor = 0,
            CursorPosition::End => model.cursor = model.query.chars().count(),
        },
        Message::MoveListSelection(direction) => match direction {
            ListDirection::Up => model.list_state.select_previous(),
            ListDirection::Down => model.list_state.select_next(),
        },
        Message::UpdateSearchQuery(update) => match update {
            QueryUpdate::Insert(c) => model.insert_char(c),
            QueryUpdate::Delete => model.delete_char_at_cursor(),
            QueryUpdate::Backspace => model.delete_char_before_cursor(),
        },
        Message::Quit => model.running = false,
    }
    None
}

/// Renders the user interface.
fn view(model: &mut Model, frame: &mut Frame) {
    let [search_area, results_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let title = Line::from("Monster Search").bold().blue().left_aligned();
    let search = Paragraph::new(model.query.as_str()).block(Block::bordered().title(title));
    frame.render_widget(search, search_area);

    // Place the terminal cursor inside the search bar (offset by the border).
    let cursor_offset = Line::from(&model.query[..model.cursor_byte_index()]).width() as u16;
    frame.set_cursor_position(Position::new(
        search_area.x + 1 + cursor_offset,
        search_area.y + 1,
    ));

    let results_title = format!("Results ({})", model.results.len());
    let results = List::new(model.results.iter().cloned())
        .block(Block::bordered().title(results_title))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    frame.render_stateful_widget(results, results_area, &mut model.list_state);

    let help = Line::from("Type to search | ↑/↓ select | Esc or Ctrl-C quit").dim();
    frame.render_widget(help, help_area);
}
