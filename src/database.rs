use chrono::DateTime;
use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqliteRow, Executor, Row, SqlitePool};
use std::{collections::BTreeSet, path::Path, sync::RwLock};

use crate::content::Article;

pub async fn create_database(path: &Path) -> sqlx::Result<SqlitePool> {
    // The pool create asynchronously
    let pool = SqlitePool::connect_lazy_with(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    );

    // TODO: Log error
    let mut trans = pool.begin().await?;

    // TODO: Database tables
    trans
        .execute(
            "CREATE TABLE IF NOT EXISTS Articles (
                id TEXT,
                source TEXT,
                title TEXT NOT NULL,
                sub_title TEXT NOT NULL,
                content TEXT NOT NULL,
                date TEXT,
                PRIMARY KEY (id, source)
            )",
        )
        .await?;
    trans.commit().await?;

    Ok(pool)
}

pub async fn get_all(pool: &SqlitePool, content: &RwLock<BTreeSet<Article>>) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let articles: Vec<Article> = sqlx::query(
        "SELECT 
            id,
            source,
            title,
            sub_title,
            content,
            date
        FROM Articles",
    )
    .try_map(|row: SqliteRow| {
        let date = if let Some(date) = row.try_get("date")? {
            DateTime::parse_from_rfc3339(date).ok()
        } else {
            None
        };
        return Ok(Article {
            id: row.try_get("id")?,
            source: row.try_get("source")?,
            title: row.try_get("title")?,
            sub_title: row.try_get("sub_title")?,
            content: row.try_get("content")?,
            date,
        });
    })
    .fetch_all(&mut conn)
    .await?;

    if articles.len() > 0 {
        let mut content = content.write().unwrap();
        for article in articles {
            content.insert(article);
        }
    }
    Ok(())
}
