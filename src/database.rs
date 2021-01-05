use sqlx::{sqlite::SqliteConnectOptions, Executor, SqlitePool};
use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, RwLock},
};

use crate::content::Article;

pub async fn create_database(path: &Path) -> sqlx::Result<Arc<SqlitePool>> {
    // The pool create asynchronously
    let pool = Arc::new(SqlitePool::connect_lazy_with(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    ));

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
                PRIMARY KEY (id, source)
            )",
        )
        .await?;
    trans.commit().await?;

    Ok(pool)
}

pub async fn get_all(
    pool: &Arc<SqlitePool>,
    content: &RwLock<HashSet<Article>>,
) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let articles: Vec<Article> = sqlx::query_as(
        "SELECT 
            id,
            source,
            title,
            sub_title,
            content
        FROM Articles",
    )
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
