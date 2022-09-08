use std::{error::Error, fmt::Debug};

use async_trait::async_trait;
use thiserror::Error;

use crate::models::ShortUrl;

pub mod memory;

#[derive(Debug, Error, PartialEq)]
pub enum RepositoryError<E: Debug + Error> {
    #[error("Underlying storage error: {0}")]
    StorageError(#[from] E),
    #[error("URL was not found")]
    NoUrlFound,
    #[error("URL already exists in the repository")]
    AlreadyExists,
}

#[async_trait]
pub trait ShortUrlRepository {
    type StorageError: Debug + Error;

    async fn get_from_short(
        &mut self,
        short: &str,
    ) -> Result<ShortUrl, RepositoryError<Self::StorageError>>;
    async fn create_shorturl(
        &mut self,
        short_url: &ShortUrl,
    ) -> Result<(), RepositoryError<Self::StorageError>>;
}
