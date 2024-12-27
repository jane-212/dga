use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to build client")]
    BuildClient,
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("failed to parse selector: {}", _0)]
    Parse(&'static str),
    #[error("join error")]
    Tokio(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;
