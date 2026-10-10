use crate::app::bestiary::BestiaryMessage;
use crate::dnd_api_client::ClientError;
use crate::model::Model;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::time::Duration;
use thiserror::Error;

pub mod bestiary;
pub mod combat;

#[derive(Error, Debug)]
pub enum DnDTuiError {
    #[error("failed to draw to the terminal")]
    TerminalError(#[from] std::io::Error),

    #[error("failed to load data from source")]
    ClientError(#[from] ClientError),
}

type Result<T> = std::result::Result<T, DnDTuiError>;

#[derive(PartialEq, Debug)]
enum Message {
    Quit,
    Bestiary(BestiaryMessage),
}

#[derive(Debug)]
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
                next = Self::update(&mut self.model, msg)?;
            }
        }
        Ok(())
    }

    fn handle_key() -> Result<Option<Message>> {
        if !event::poll(Duration::from_millis(250))? {
            return Ok(None);
        }
        let Event::Key(key) = event::read()? else {
            return Ok(None);
        };
        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }
        Ok(match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c' | 'C')) => Some(Message::Quit),
            _ => bestiary::handle_key(key).map(Message::Bestiary),
        })
    }

    fn update(model: &mut Model, msg: Message) -> Result<Option<Message>> {
        Ok(match msg {
            Message::Quit | Message::Bestiary(BestiaryMessage::Leave) => {
                model.running = false;
                None
            }
            Message::Bestiary(BestiaryMessage::RequestMonsterById(id)) => {
                let monster = model.api_client.monster_by_id(&id)?;
                model.bestiary_model.monsters.insert(id.clone(), monster);
                Some(Message::Bestiary(BestiaryMessage::OpenMonsterDetails(id)))
            }
            Message::Bestiary(msg) => {
                bestiary::update(&mut model.bestiary_model, msg).map(Message::Bestiary)
            }
        })
    }
}
