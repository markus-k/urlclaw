use async_trait::async_trait;

use crate::models::ShortUrl;
use crate::repository::ShortUrlRepository;
use crate::UrlclawError;

#[derive(Debug, Default)]
pub struct InMemoryRepository {
    urls: Vec<ShortUrl>,
}

#[async_trait]
impl ShortUrlRepository for InMemoryRepository {
    async fn get_from_short(&mut self, short: &str) -> Result<ShortUrl, UrlclawError> {
        let short_urls = self
            .urls
            .iter()
            .filter(|short_url| short_url.short_url().as_str() == short)
            .collect::<Vec<_>>();

        if short_urls.is_empty() {
            Err(UrlclawError::UrlNotFound)
        } else {
            Ok(short_urls.first().unwrap().to_owned().clone())
        }
    }

    async fn create_shorturl(&mut self, short_url: &ShortUrl) -> Result<(), UrlclawError> {
        if self
            .urls
            .iter()
            .filter(|s| s.short_url() == short_url.short_url())
            .count()
            == 0
        {
            self.urls.push(short_url.clone());
            Ok(())
        } else {
            Err(UrlclawError::ShortAlreadyExists)
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
