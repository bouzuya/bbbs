#[derive(Clone)]
pub struct SqliteStore(sqlx::SqlitePool);

impl SqliteStore {
    pub async fn new() -> Self {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite:./db.sqlite?mode=rwc")
            .await
            .unwrap();
        sqlx::query(
            r#"
CREATE TABLE IF NOT EXISTS thread_event_streams (
    id      TEXT    NOT NULL    PRIMARY KEY,
    version INTEGER NOT NULL
)"#,
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
CREATE TABLE IF NOT EXISTS thread_events (
    id          TEXT    NOT NULL   PRIMARY KEY,
    thread_id   TEXT    NOT NULL,
    content     TEXT    NOT NULL, 
    version     INTEGER NOT NULL,
    at          INTEGER NOT NULL,
    UNIQUE (thread_id, version)
)"#,
        )
        .execute(&pool)
        .await
        .unwrap();
        Self(pool)
    }
}

impl crate::store::Store for SqliteStore {}

#[async_trait::async_trait]
impl crate::port::ThreadReader for SqliteStore {
    async fn get_thread(
        &self,
        _id: &crate::model::shared::id::ThreadId,
    ) -> Option<crate::model::read::Thread> {
        todo!()
    }

    async fn list_threads(&self) -> Vec<crate::model::read::Thread> {
        todo!()
    }
}

#[async_trait::async_trait]
impl crate::port::ThreadRepository for SqliteStore {
    async fn find(
        &self,
        _id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        todo!()
    }

    async fn store(
        &self,
        _version: Option<crate::model::write::Version>,
        _events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new() {
        let _store = SqliteStore::new().await;
    }
}
