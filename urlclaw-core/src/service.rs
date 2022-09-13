use crate::models::ShortUrl;
use crate::repository::ShortUrlRepository;
use crate::UrlclawError;

pub async fn create_shorturl<R: ShortUrlRepository>(
    repository: &mut R,
    short: String,
    target: String,
) -> Result<ShortUrl, UrlclawError> {
    let short_url = ShortUrl::new(short, target.parse()?)?;

    repository.create_shorturl(&short_url).await?;

    Ok(short_url)
}

pub async fn get_shorturl_target<R: ShortUrlRepository>(
    repository: &mut R,
    short: &str,
) -> Result<ShortUrl, UrlclawError> {
    let short_url = repository.get_from_short(short).await?;

    Ok(short_url)
}

#[cfg(test)]
mod tests {
    use crate::repository::memory::InMemoryRepository;

    use super::*;

    #[tokio::test]
    async fn test_service_create_shorturl() {
        let mut repo = InMemoryRepository::default();
        let short_url = create_shorturl(
            &mut repo,
            "rust".to_owned(),
            "https://rust-lang.org".to_owned(),
        )
        .await
        .unwrap();

        assert_eq!(short_url.short_url().as_str(), "rust");
        assert_eq!(short_url.target_url().as_str(), "https://rust-lang.org/");

        // assert existing URLs aren't overwritten
        assert!(matches!(
            create_shorturl(
                &mut repo,
                "rust".to_owned(),
                "https://example.org".to_owned(),
            )
            .await,
            Err(UrlclawError::ShortAlreadyExists)
        ));
    }

    #[tokio::test]
    async fn test_service_get_shorturl() {
        let mut repo = InMemoryRepository::default();

        repo.create_shorturl(
            &ShortUrl::new("test".to_owned(), "https://example.com/".parse().unwrap()).unwrap(),
        )
        .await
        .unwrap();

        let short_url = get_shorturl_target(&mut repo, "test").await.unwrap();
        assert_eq!(short_url.short_url().as_str(), "test");
        assert_eq!(short_url.target_url().as_str(), "https://example.com/");
    }

    #[tokio::test]
    async fn test_service_get_shorturl_notfound() {
        let mut repo = InMemoryRepository::default();

        // assert non-existing URL returns not found
        assert!(matches!(
            get_shorturl_target(&mut repo, "rust").await,
            Err(UrlclawError::UrlNotFound)
        ));
    }
}
