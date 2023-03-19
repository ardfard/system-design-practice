use async_trait::async_trait;
use sqlx::sqlite::SqlitePoolOptions;
use url::{ParseError, Url};

#[async_trait]
pub trait UrlShortener {
    async fn shorten(&self, url: Url) -> Result<String, sqlx::Error>;
    async fn get_real_url(&self, id: i64) -> Result<String, sqlx::Error>;
}

pub struct IncrementalUrlShortener {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl IncrementalUrlShortener {
    pub async fn new(conn_string: &str) -> Result<IncrementalUrlShortener, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(conn_string)
            .await?;
        Ok(IncrementalUrlShortener { pool })
    }
}

#[async_trait]
impl UrlShortener for IncrementalUrlShortener {
    async fn shorten(&self, url: Url) -> Result<String, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let url_string = url.as_str();
        let id = sqlx::query!(
            r#"
            INSERT INTO urls (url)
            VALUES (?1)
         "#,
            url_string
        )
        .execute(&mut conn)
        .await?
        .last_insert_rowid();

        Ok(base62::encode(id as u64))
    }

    async fn get_real_url(&self, id: i64) -> Result<String, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let url = sqlx::query!(
            r#"
            SELECT url FROM urls
            WHERE id = ?1
         "#,
            id
        )
        .fetch_one(&mut conn)
        .await?
        .url;

        url.ok_or(sqlx::Error::RowNotFound)
    }
}

#[cfg(test)]
mod tests {
    use crate::{IncrementalUrlShortener, UrlShortener};
    use url::Url;

    #[tokio::test]
    async fn it_can_shorten_url() {
        let shortener = IncrementalUrlShortener::new("sqlite:test.db")
            .await
            .unwrap();
        let short_path = shortener
            .shorten(Url::parse("https://gamefaqs.com").unwrap())
            .await;

        assert!(short_path.is_ok());
    }

    #[test]
    fn repl() {
        let test = base62::encode(100000000u64);
        println!("{test:?}");
    }
}
