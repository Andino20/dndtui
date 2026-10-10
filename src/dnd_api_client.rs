use crate::monster::{
    AbilityScores, Action, ArmorClass, ArmorClassSource, Monster, MonsterEntry, Speed,
};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to request data from api")]
    HttpClientError(#[from] reqwest::Error),
}

type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Deserialize)]
struct MonsterListResponse {
    results: Vec<MonsterEntryResponse>,
}

#[derive(Debug, Deserialize)]
struct MonsterEntryResponse {
    index: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MonsterResponse {
    name: String,
    size: String,
    #[serde(rename = "type")]
    monster_type: String,
    challenge_rating: f32,
    alignment: String,
    armor_class: Vec<ArmorClassResponse>,
    hit_points: u32,
    hit_points_roll: String,
    speed: SpeedResponse,
    strength: u8,
    dexterity: u8,
    constitution: u8,
    intelligence: u8,
    wisdom: u8,
    charisma: u8,
    actions: Vec<ActionResponse>,
}

/// Which of the optional fields is present depends on `type`.
#[derive(Debug, Deserialize)]
struct ArmorClassResponse {
    #[serde(rename = "type")]
    armor_class_type: String,
    value: u8,
    armor: Option<Vec<ReferenceResponse>>,
    spell: Option<ReferenceResponse>,
    condition: Option<ReferenceResponse>,
    desc: Option<String>,
}

/// A link to another resource of the API.
#[derive(Debug, Deserialize)]
struct ReferenceResponse {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SpeedResponse {
    walk: Option<String>,
    burrow: Option<String>,
    climb: Option<String>,
    fly: Option<String>,
    swim: Option<String>,
    #[serde(default)]
    hover: bool,
}

#[derive(Debug, Deserialize)]
struct ActionResponse {
    name: String,
    desc: String,
}

impl From<MonsterResponse> for Monster {
    fn from(value: MonsterResponse) -> Self {
        Monster {
            name: value.name,
            size: value.size,
            monster_type: capitalize(value.monster_type),
            challenge_rating: value.challenge_rating,
            alignment: value.alignment,
            armor_class: value.armor_class.into_iter().map(Into::into).collect(),
            hit_points: value.hit_points,
            hit_points_roll: value.hit_points_roll,
            speed: value.speed.into(),
            ability_scores: AbilityScores {
                strength: value.strength,
                dexterity: value.dexterity,
                constitution: value.constitution,
                intelligence: value.intelligence,
                wisdom: value.wisdom,
                charisma: value.charisma,
            },
            actions: value.actions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ArmorClassResponse> for ArmorClass {
    fn from(value: ArmorClassResponse) -> Self {
        let source = match value.armor_class_type.as_str() {
            "dex" => ArmorClassSource::Dexterity,
            "natural" => ArmorClassSource::Natural,
            // Improvised armor has no equipment entries and is only described in `desc`.
            "armor" => ArmorClassSource::Armor(match value.armor {
                Some(armor) => armor.into_iter().map(|a| a.name).collect(),
                None => value.desc.into_iter().collect(),
            }),
            "spell" => ArmorClassSource::Spell(value.spell.map(|s| s.name).unwrap_or_default()),
            "condition" => {
                ArmorClassSource::Condition(value.condition.map(|c| c.name).unwrap_or_default())
            }
            other => ArmorClassSource::Other(value.desc.unwrap_or_else(|| other.to_string())),
        };
        ArmorClass {
            value: value.value,
            source,
        }
    }
}

impl From<SpeedResponse> for Speed {
    fn from(value: SpeedResponse) -> Self {
        Speed {
            walk: value.walk,
            burrow: value.burrow,
            climb: value.climb,
            fly: value.fly,
            swim: value.swim,
            hover: value.hover,
        }
    }
}

impl From<ActionResponse> for Action {
    fn from(value: ActionResponse) -> Self {
        Action {
            name: value.name,
            description: value.desc,
        }
    }
}

#[derive(Debug)]
pub struct DndApiClient {
    client: Client,
    base_url: String,
}

impl DndApiClient {
    pub fn new(base_url: &str) -> Result<Self> {
        // The API redirects plain HTTP to HTTPS in production mode unless a proxy vouches for the
        // request.
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));

        Ok(DndApiClient {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .default_headers(headers)
                .build()?,
            base_url: base_url.to_string(),
        })
    }

    pub fn all_monsters(&self) -> Result<Vec<MonsterEntry>> {
        let monsters_response = self.get::<MonsterListResponse>("/monsters")?.results;
        Ok(monsters_response
            .into_iter()
            .map(|response| MonsterEntry {
                id: response.index,
                name: response.name,
            })
            .collect())
    }

    pub fn monster_by_id(&self, id: &str) -> Result<Monster> {
        let response = self.get::<MonsterResponse>(&format!("/monsters/{id}"))?;
        Ok(response.into())
    }

    fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        Ok(self
            .client
            .get(format!("{}{}", self.base_url, url))
            .send()?
            .json()?)
    }
}

fn capitalize(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
