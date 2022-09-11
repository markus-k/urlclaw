use async_trait::async_trait;

use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

use crate::models::ShortUrl;
use crate::repository::ShortUrlRepository;
use crate::UrlclawError;

#[derive(Clone, Debug)]
pub struct SqlxRepository {
    pool: PgPool,
}

impl SqlxRepository {
    pub async fn new(db_url: &str) -> Result<Self, UrlclawError> {
        let pool = PgPoolOptions::new().connect(db_url).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!().run(&self.pool).await?;

        Ok(())
    }
}

#[async_trait]
impl ShortUrlRepository for SqlxRepository {
    async fn get_from_short(&mut self, short: &str) -> Result<ShortUrl, UrlclawError> {
        let row: (Uuid, String, String) =
            match sqlx::query_as("SELECT id, short, target FROM short_urls WHERE short = $1")
                .bind(short)
                .fetch_one(&self.pool)
                .await
            {
                Ok(row) => Ok(row),
                Err(sqlx::Error::RowNotFound) => Err(UrlclawError::UrlNotFound),
                Err(e) => Err(UrlclawError::Database(e)),
            }?;

        Ok(ShortUrl::from_db(row.0, row.1, row.2)?)
    }

    async fn create_shorturl(&mut self, short_url: &ShortUrl) -> Result<(), UrlclawError> {
        match self.get_from_short(short_url.short_url().as_str()).await {
            Err(UrlclawError::UrlNotFound) => {
                sqlx::query("INSERT INTO short_urls (id, short, target) VALUES ($1, $2, $3)")
                    .bind(short_url.uuid())
                    .bind(&short_url.short_url().as_str())
                    .bind(&short_url.target_url().to_string())
                    .execute(&self.pool)
                    .await?;
                Ok(())
            }
            Ok(_) => Err(UrlclawError::ShortAlreadyExists),
            Err(e) => Err(e),
        }
    }
}
