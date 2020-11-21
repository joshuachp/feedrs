use sqlx::SqlitePool;

pub fn create_database(uri: &str) -> SqlitePool {
    tokio::spawn(async move {
        //let conn = SqliteConnection::connect(&uri).await.unwrap();
    });
    todo!();
}
