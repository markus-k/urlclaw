use thiserror::Error;

pub mod models;
pub mod repository;
pub mod service;

#[derive(Debug, Error)]
pub enum UrlclawError {
    #[error("The short URL contains invalid characters")]
    ShortUrlInvalid,
    #[error("Short URL with given short already exists")]
    ShortAlreadyExists,
    #[error("Short URL was not found")]
    UrlNotFound,
    #[error("Invalid target URL: {0}")]
    InvalidTarget(#[from] url::ParseError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
