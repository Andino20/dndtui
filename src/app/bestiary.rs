use crate::app;
use crate::app::Message;
use crate::app::Message::{Bestiary, RequestMonsterById};
use crate::app::Result;
use crate::app::bestiary::BestiaryMessage::{EditSearch, MoveListSelection};
use crate::dnd_api_client::DndApiClient;
use crate::monster::{
    AbilityScores, Action, ArmorClass, ArmorClassSource, Monster, MonsterEntry, Speed,
};
use BestiaryMessage::EnterMonsterEntry;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Modifier, Span, Style};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, List, ListState, Paragraph, Row, Table, Wrap};
use std::collections::HashMap;
use tui_input::backend::crossterm::to_input_request;
use tui_input::{Input, InputRequest};

#[derive(Debug, Default)]
pub struct BestiaryModel {
    /// The search bar.
    pub search: Input,
    /// Every monster known to the API, loaded once at startup.
    pub all_monsters: Vec<MonsterEntry>,
    /// Cached monster details requested from the API during runtime.
    pub monsters: HashMap<String, Monster>,
    /// Indices into `all_monsters` of the monsters matching the current query.
    pub results: Vec<usize>,
    /// Selection and scroll state of the result list.
    pub list_state: ListState,

    pub detailed_monster_id: Option<String>,
}

#[derive(PartialEq, Debug)]
pub enum BestiaryMessage {
    EnterMonsterEntry,
    OpenMonsterDetails(String),
    MoveListSelection(ListDirection),
    EditSearch(InputRequest),
}

#[derive(PartialEq, Debug)]
pub enum ListDirection {
    Up,
    Down,
}

impl BestiaryModel {
    pub fn new(api_client: &DndApiClient) -> Result<BestiaryModel> {
        let mut model = Self { ..Self::default() };
        model.all_monsters = api_client.all_monsters()?;
        model.update_results();
        Ok(model)
    }

    pub fn update_results(&mut self) {
        let query = self.search.value().to_lowercase();
        self.results = self
            .all_monsters
            .iter()
            .enumerate()
            .filter(|(_, monster)| monster.name.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
        self.list_state
            .select((!self.results.is_empty()).then_some(0));
    }
}

pub fn update(model: &mut BestiaryModel, msg: BestiaryMessage) -> Result<Option<Message>> {
    match msg {
        EnterMonsterEntry => {
            // can probably split this update up a bit (check cache -> request monster -> open details)
            if let Some(selected) = model.list_state.selected() {
                let selected_monster_entry_id = &model.all_monsters[model.results[selected]].id;
                if !model.monsters.contains_key(selected_monster_entry_id) {
                    return Ok(Some(RequestMonsterById(
                        selected_monster_entry_id.to_string(),
                    )));
                }

                return Ok(Some(Bestiary(BestiaryMessage::OpenMonsterDetails(
                    selected_monster_entry_id.to_string(),
                ))));
            }
        }
        BestiaryMessage::OpenMonsterDetails(monster_id) => {
            model.detailed_monster_id = Some(monster_id)
        }
        MoveListSelection(direction) => match direction {
            ListDirection::Up => model.list_state.select_previous(),
            ListDirection::Down => model.list_state.select_next(),
        },
        EditSearch(request) => {
            let query_changed = model
                .search
                .handle(request)
                .is_some_and(|change| change.value);
            if query_changed {
                model.update_results();
            }
        }
        _ => {}
    }
    Ok(None)
}

pub fn handle_key(event: Event) -> app::Result<Option<Message>> {
    if let Event::Key(key) = event
        && key.kind == KeyEventKind::Press
    {
        return Ok(match (key.modifiers, key.code) {
            (_, KeyCode::Down) => Some(Bestiary(MoveListSelection(ListDirection::Down))),
            (_, KeyCode::Up) => Some(Bestiary(MoveListSelection(ListDirection::Up))),
            (_, KeyCode::Enter) => Some(Bestiary(EnterMonsterEntry)),
            _ => to_input_request(&event).map(|request| Bestiary(EditSearch(request))),
        });
    }
    Ok(None)
}

pub fn view(model: &mut BestiaryModel, frame: &mut Frame, area: Rect) {
    let [search_area, content_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
        .areas(area);

    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(content_area);

    // Two columns for the border, one to keep the cursor visible at the end of the query.
    let width = search_area.width.saturating_sub(3);
    let scroll = model.search.visual_scroll(width as usize);

    let title = Line::from(" Monster Search ").bold().blue().left_aligned();
    let search = Paragraph::new(model.search.value())
        .scroll((0, scroll as u16))
        .block(Block::bordered().title(title));
    frame.render_widget(search, search_area);

    let cursor = model.search.visual_cursor().saturating_sub(scroll) + 1;
    frame.set_cursor_position((search_area.x + cursor as u16, search_area.y + 1));

    let results_title = format!(" Results ({}) ", model.results.len());
    let names = model
        .results
        .iter()
        .map(|&index| model.all_monsters[index].name.as_str());
    let results = List::new(names)
        .block(Block::bordered().title(results_title))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    frame.render_stateful_widget(
        results,
        if model.detailed_monster_id.is_some() {
            list_area
        } else {
            content_area
        },
        &mut model.list_state,
    );

    if let Some(monster_id) = &model.detailed_monster_id {
        let monster = model.monsters.get(monster_id).unwrap();
        render_monster_details(monster, frame, detail_area);
    }

    let help = Line::from("Type to search | ↑/↓ select | Esc or Ctrl-C quit or close a page").dim();
    frame.render_widget(help, help_area);
}

fn render_monster_details(monster: &Monster, frame: &mut Frame, area: Rect) {
    let block = Block::bordered().title(" Monster Details ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let summary = vec![
        Line::from(Span::styled(&monster.name, Style::default().bold())),
        Line::from(format!(
            "{} {}, {}",
            monster.size, monster.monster_type, monster.alignment
        ))
            .dim(),
        Line::default(),
        labeled("AC", format_armor_class(&monster.armor_class)),
        labeled("Initiative", format!("{:+}", monster.initiative())),
        labeled(
            "HP",
            format!("{} ({})", monster.hit_points, monster.hit_points_roll),
        ),
        labeled("Speed", format_speed(&monster.speed)),
        labeled("CR", monster.challenge_rating.to_string()),
    ];

    // One line below the summary and below the table to separate the sections.
    let [summary_area, stats_area, actions_area] = Layout::vertical([
        Constraint::Length(summary.len() as u16 + 1),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
        .areas(inner);

    frame.render_widget(Paragraph::new(summary), summary_area);

    let scores = &monster.ability_scores;
    let stats = [
        scores.strength,
        scores.dexterity,
        scores.constitution,
        scores.intelligence,
        scores.wisdom,
        scores.charisma,
    ]
        .map(|score| format!("{} ({:+})", score, AbilityScores::modifier(score)));
    let stats_table = Table::new([Row::new(stats)], [Constraint::Fill(1); 6])
        .header(Row::new(["STR", "DEX", "CON", "INT", "WIS", "CHA"]).bold());
    frame.render_widget(stats_table, stats_area);

    let actions = Paragraph::new(action_lines(&monster.actions)).wrap(Wrap { trim: false });
    frame.render_widget(actions, actions_area);
}

/// Lays out each action as its bold name followed by the description, with an empty line between
/// actions. A description can span several paragraphs, which each start on a new line.
fn action_lines(actions: &[Action]) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    for action in actions {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        let mut paragraphs = action
            .description
            .lines()
            .filter(|paragraph| !paragraph.trim().is_empty());
        lines.push(Line::from(vec![
            Span::styled(format!("{}.", action.name), Style::default().bold()),
            Span::raw(" "),
            Span::raw(paragraphs.next().unwrap_or_default()),
        ]));
        lines.extend(paragraphs.map(Line::from));
    }
    lines
}

fn labeled(label: &'static str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().bold()),
        Span::raw(" "),
        Span::raw(value),
    ])
}

/// Formats armor classes the way stat blocks do, such as `12, 15 with mage armor`.
fn format_armor_class(armor_class: &[ArmorClass]) -> String {
    armor_class
        .iter()
        .map(|ac| match &ac.source {
            ArmorClassSource::Dexterity => ac.value.to_string(),
            ArmorClassSource::Natural => format!("{} (natural armor)", ac.value),
            ArmorClassSource::Armor(pieces) if pieces.is_empty() => ac.value.to_string(),
            ArmorClassSource::Armor(pieces) => {
                format!("{} ({})", ac.value, pieces.join(", ").to_lowercase())
            }
            ArmorClassSource::Spell(spell) => {
                format!("{} with {}", ac.value, spell.to_lowercase())
            }
            ArmorClassSource::Condition(condition) => {
                format!("{} while {}", ac.value, condition.to_lowercase())
            }
            ArmorClassSource::Other(description) => format!("{} ({})", ac.value, description),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Formats speeds the way stat blocks do, such as `30 ft., fly 60 ft. (hover)`.
fn format_speed(speed: &Speed) -> String {
    let mut parts = Vec::new();
    if let Some(walk) = &speed.walk {
        parts.push(walk.clone());
    }
    if let Some(burrow) = &speed.burrow {
        parts.push(format!("burrow {burrow}"));
    }
    if let Some(climb) = &speed.climb {
        parts.push(format!("climb {climb}"));
    }
    if let Some(fly) = &speed.fly {
        let hover = if speed.hover { " (hover)" } else { "" };
        parts.push(format!("fly {fly}{hover}"));
    }
    if let Some(swim) = &speed.swim {
        parts.push(format!("swim {swim}"));
    }
    parts.join(", ")
}
