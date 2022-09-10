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
