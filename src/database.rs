use sqlx::Executor;
use sqlx::SqlitePool;
use std::sync::Arc;

pub fn create_database(uri: &str) -> sqlx::Result<Arc<SqlitePool>> {
    let pool = Arc::new(SqlitePool::connect_lazy(uri)?);

    let t_pool = Arc::clone(&pool);
    tokio::spawn(async move {
        // TODO: Log error
        let mut trans = t_pool.begin().await.unwrap();
        // TODO: Database tables
        trans.execute("CREATE TABLE IF NOT EXISTS RSS ()");
        trans.execute("CREATE TABLE IF NOT EXISTS Atom ()");
        //let conn = SqliteConnection::connect(&uri).await.unwrap();
        // TODO: Check result
        trans.commit().await.unwrap();
    });

    Ok(pool)
}
