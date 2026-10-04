use crate::dnd_api_client::ClientError;
use crate::{ListDirection, Message, Model, ui};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::time::Duration;
use thiserror::Error;
use tui_input::backend::crossterm::to_input_request;

#[derive(Error, Debug)]
pub enum DnDTuiError {
    #[error("failed to draw to the terminal")]
    TerminalError(#[from] std::io::Error),

    #[error("failed to load data from source")]
    ClientError(#[from] ClientError),
}

type Result<T> = std::result::Result<T, DnDTuiError>;

#[derive(Debug, Default)]
pub struct App {
    model: Model,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(App {
            model: Model::new()?,
        })
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.model.running = true;
        while self.model.running {
            terminal.draw(|frame| ui::view(&mut self.model, frame))?;

            let mut next = Self::handle_key()?;
            while let Some(msg) = next {
                next = Self::update(&mut self.model, msg);
            }
        }
        Ok(())
    }

    fn handle_key() -> Result<Option<Message>> {
        if !event::poll(Duration::from_millis(250))? {
            return Ok(None);
        }
        let event = event::read()?;
        let Event::Key(key) = &event else {
            return Ok(None);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        Ok(match (key.modifiers, key.code) {
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c' | 'C')) => {
                Some(Message::Quit)
            }
            (_, KeyCode::Down) => Some(Message::MoveListSelection(ListDirection::Down)),
            (_, KeyCode::Up) => Some(Message::MoveListSelection(ListDirection::Up)),
            _ => to_input_request(&event).map(Message::EditSearch),
        })
    }

    fn update(model: &mut Model, msg: Message) -> Option<Message> {
        match msg {
            Message::EditSearch(request) => {
                let query_changed = model
                    .search
                    .handle(request)
                    .is_some_and(|change| change.value);
                if query_changed {
                    model.update_results();
                }
            }
            Message::MoveListSelection(ListDirection::Up) => model.list_state.select_previous(),
            Message::MoveListSelection(ListDirection::Down) => model.list_state.select_next(),
            Message::Quit => model.running = false,
        }
        None
    }
}
