use sqlx::{sqlite::SqliteConnectOptions, Executor, SqlitePool};
use std::{collections::BTreeSet, path::Path, sync::RwLock};

use crate::content::Article;

macro_rules! user_version {
    () => {
        1
    };
}

// This will delete and not migrate the database if the version is changed, since is used only as
// cache
pub async fn get_database(path: &Path) -> sqlx::Result<SqlitePool> {
    // The pool create asynchronously
    let pool = SqlitePool::connect_lazy_with(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    );

    let mut conn = pool.acquire().await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version;")
        .fetch_one(&mut conn)
        .await?;
    dbg!(version);

    if version != user_version!() {
        if version != 0 {
            dbg!("Delete");
            delete_database(&pool).await?;
        }
        dbg!("create");
        create_database(&pool).await?;
    }

    Ok(pool)
}

pub async fn create_database(pool: &SqlitePool) -> sqlx::Result<()> {
    let mut trans = pool.begin().await?;
    trans
        .execute(concat!("PRAGMA user_version = ", user_version!(), ";"))
        .await?;
    trans
        .execute(
            "CREATE TABLE IF NOT EXISTS Articles (
                id TEXT NOT NULL,
                source TEXT NOT NULL,
                title TEXT NOT NULL,
                sub_title TEXT NOT NULL,
                content TEXT NOT NULL,
                date DATETIME,
                PRIMARY KEY (id, source)
            )",
        )
        .await?;
    trans.commit().await?;
    Ok(())
}

pub async fn delete_database(pool: &SqlitePool) -> sqlx::Result<()> {
    let mut trans = pool.begin().await?;
    trans.execute("DROP TABLE IF EXISTS Articles");
    trans.commit().await?;
    Ok(())
}

pub async fn get_all(pool: &SqlitePool, content: &RwLock<BTreeSet<Article>>) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;
    let articles: Vec<Article> = sqlx::query_as(
        "SELECT 
            id,
            source,
            title,
            sub_title,
            content,
            date
        FROM Articles",
    )
    .fetch_all(&mut conn)
    .await?;

    if !articles.is_empty() {
        let mut content = content.write().unwrap();
        for article in articles {
            content.insert(article);
        }
    }
    Ok(())
}
