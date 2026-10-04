use crate::repository_error::RepositoryError;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::time::Duration;

pub struct DnDRepository<T: DeserializeOwned> {
    base_url: String,
    _marker: PhantomData<T>,
}

impl<T: DeserializeOwned> DnDRepository<T> {
    pub fn new(base_url: &str) -> DnDRepository<T> {
        DnDRepository {
            base_url: base_url.to_string(),
            _marker: Default::default(),
        }
    }

    pub fn get(&self) -> Result<T, RepositoryError> {
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(Duration::new(5, 0))
            .build()?;
        Ok(client.get(self.base_url.as_str()).send()?.json()?)
    }
}
