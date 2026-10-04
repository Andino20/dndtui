use crate::monster::Monster;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
use thiserror::Error;

const MONSTERS_BASE_URL: &str = "http://localhost:3000/api/2014/monsters";

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to request data from api")]
    HttpClientError(#[from] reqwest::Error),
}

type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Deserialize)]
struct MonsterListResponse {
    results: Vec<MonsterResponse>,
}

#[derive(Debug, Deserialize)]
struct MonsterResponse {
    name: String,
}

pub fn all_monsters() -> Result<Vec<Monster>> {
    let monsters_response = get::<MonsterListResponse>(MONSTERS_BASE_URL)?.results;
    Ok(monsters_response
        .into_iter()
        .map(|response| Monster {
            name: response.name,
        })
        .collect())
}

fn get<T: DeserializeOwned>(url: &str) -> Result<T> {
    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(Duration::new(5, 0))
        .build()?;
    Ok(client.get(url).send()?.json()?)
}
