use async_trait::async_trait;

use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;

use crate::models::ShortUrl;
use crate::repository::{RepositoryError, ShortUrlRepository};

#[derive(Debug)]
pub struct SqlxRepository {
    pool: PgPool,
}

impl SqlxRepository {
    pub async fn new(db_url: &str) -> Result<Self, RepositoryError<sqlx::Error>> {
        let pool = PgPoolOptions::new().connect(db_url).await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), RepositoryError<sqlx::migrate::MigrateError>> {
        sqlx::migrate!().run(&self.pool).await?;

        Ok(())
    }
}

#[async_trait]
impl ShortUrlRepository for SqlxRepository {
    type StorageError = sqlx::Error;

    async fn get_from_short(
        &mut self,
        short: &str,
    ) -> Result<ShortUrl, RepositoryError<Self::StorageError>> {
        let row: (Uuid, String, String) =
            match sqlx::query_as("SELECT id, short, target FROM short_urls WHERE short = $1")
                .bind(short)
                .fetch_one(&self.pool)
                .await
            {
                Ok(row) => Ok(row),
                Err(sqlx::Error::RowNotFound) => Err(RepositoryError::NoUrlFound),
                Err(e) => Err(RepositoryError::StorageError(e)),
            }?;

        Ok(ShortUrl {
            id: row.0,
            short: row.1,
            target: row.2,
        })
    }

    async fn create_shorturl(
        &mut self,
        short_url: &ShortUrl,
    ) -> Result<(), RepositoryError<Self::StorageError>> {
        match self.get_from_short(&short_url.short).await {
            Err(RepositoryError::NoUrlFound) => {
                sqlx::query("INSERT INTO short_urls (id, short, target) VALUES ($1, $2, $3)")
                    .bind(short_url.id)
                    .bind(&short_url.short)
                    .bind(&short_url.target)
                    .execute(&self.pool)
                    .await?;
                Ok(())
            }
            Ok(_) => Err(RepositoryError::AlreadyExists),
            Err(e) => Err(e),
        }
    }
}
