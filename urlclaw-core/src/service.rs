use thiserror::Error;

use crate::models::ShortUrl;
use crate::repository::{RepositoryError, ShortUrlRepository};

#[derive(Debug, Error)]
pub enum ServiceError<R: ShortUrlRepository>
where
    <R as ShortUrlRepository>::StorageError: 'static,
{
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError<R::StorageError>),
}

pub async fn create_shorturl<R: ShortUrlRepository>(
    repository: &mut R,
    short: String,
    target: String,
) -> Result<ShortUrl, ServiceError<R>>
where
    <R as ShortUrlRepository>::StorageError: 'static,
{
    let short_url = ShortUrl::new(short, target.parse().unwrap()).unwrap();

    repository.create_shorturl(&short_url).await?;

    Ok(short_url)
}

pub async fn get_shorturl_target<R: ShortUrlRepository>(
    repository: &mut R,
    short: &str,
) -> Result<ShortUrl, ServiceError<R>>
where
    <R as ShortUrlRepository>::StorageError: 'static,
{
    let short_url = repository.get_from_short(short).await?;

    Ok(short_url)
}
