use async_trait::async_trait;
use thiserror::Error;

use crate::models::ShortUrl;
use crate::repository::{RepositoryError, ShortUrlRepository};

#[derive(Debug, Error, PartialEq)]
pub enum InMemoryError {
    #[error("The given short url is not unique in the repository.")]
    NotUnique,
}

#[derive(Debug)]
pub struct InMemoryRepository {
    urls: Vec<ShortUrl>,
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self { urls: Vec::new() }
    }
}

#[async_trait]
impl ShortUrlRepository for InMemoryRepository {
    type StorageError = InMemoryError;

    async fn get_from_short(
        &mut self,
        short: &str,
    ) -> Result<ShortUrl, RepositoryError<Self::StorageError>> {
        let short_urls = self
            .urls
            .iter()
            .filter(|short_url| short_url.short_url() == short)
            .collect::<Vec<_>>();

        if short_urls.len() > 1 {
            Err(RepositoryError::StorageError(InMemoryError::NotUnique))
        } else if short_urls.len() == 0 {
            Err(RepositoryError::NoUrlFound)
        } else {
            Ok(short_urls.first().unwrap().to_owned().clone())
        }
    }

    async fn create_shorturl(
        &mut self,
        short_url: &ShortUrl,
    ) -> Result<(), RepositoryError<Self::StorageError>> {
        if self
            .urls
            .iter()
            .filter(|s| s.short_url() == short_url.short_url())
            .count()
            == 0
        {
            Ok(self.urls.push(short_url.clone()))
        } else {
            Err(RepositoryError::AlreadyExists)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inmemory_repo() {
        let mut repo = InMemoryRepository::default();

        let short_url =
            ShortUrl::new("rust".to_owned(), "https://rust-lang.org".parse().unwrap()).unwrap();

        repo.create_shorturl(&short_url).await.unwrap();

        let other_short_url = repo.get_from_short("rust").await.unwrap();
        assert_eq!(short_url, other_short_url);
    }
}
