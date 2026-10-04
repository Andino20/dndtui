use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("failed to request data from repository")]
    RequestFailed(#[from] reqwest::Error),
}
