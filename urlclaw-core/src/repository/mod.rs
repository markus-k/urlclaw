use crate::{models::ShortUrl, UrlclawError};
use async_trait::async_trait;

pub mod memory;
pub mod sqlx;

#[async_trait]
pub trait ShortUrlRepository {
    async fn get_from_short(&mut self, short: &str) -> Result<ShortUrl, UrlclawError>;
    async fn create_shorturl(&mut self, short_url: &ShortUrl) -> Result<(), UrlclawError>;
}
