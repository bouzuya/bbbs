use sqlx::Row as _;

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
    at          INTEGER NOT NULL,
    content     TEXT    NOT NULL,
    id          TEXT    NOT NULL   PRIMARY KEY,
    kind        TEXT    NOT NULL,
    thread_id   TEXT    NOT NULL,
    version     INTEGER NOT NULL,
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

#[derive(Debug, thiserror::Error)]
enum SqliteStoreError {
    #[error("find begin transaction")]
    FindBeginTransaction(#[source] sqlx::Error),
    #[error("find select event streams")]
    FindSelectEventStreams(#[source] sqlx::Error),
    #[error("find select events")]
    FindSelectEvents(#[source] sqlx::Error),
    #[error("store begin transaction")]
    StoreBeginTransaction(#[source] sqlx::Error),
    #[error("store commit")]
    StoreCommit(#[source] sqlx::Error),
    #[error("store insert event streams")]
    StoreInsertEventStreams(#[source] sqlx::Error),
    #[error("store insert events")]
    StoreInsertEvents(#[source] sqlx::Error),
}

impl From<SqliteStoreError> for crate::port::ThreadRepositoryError {
    fn from(err: SqliteStoreError) -> Self {
        // TODO
        Self::InternalError(err.into())
    }
}

#[async_trait::async_trait]
impl crate::port::ThreadRepository for SqliteStore {
    async fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        self.0
            .begin()
            .await
            .map_err(SqliteStoreError::FindBeginTransaction)?;
        let mut tx = self
            .0
            .acquire()
            .await
            .map_err(SqliteStoreError::FindBeginTransaction)?;
        let row = sqlx::query(include_str!("sqlite_store/select_thread_event_streams.sql"))
            .bind(id.to_string())
            .fetch_optional(&mut *tx)
            .await
            .map_err(SqliteStoreError::FindSelectEventStreams)?;
        match row {
            None => Ok(None),
            Some(_) => {
                let events = sqlx::query(include_str!("sqlite_store/select_thread_events.sql"))
                    .bind(id.to_string())
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(SqliteStoreError::FindSelectEvents)
                    .map(|events| {
                        events
                            .into_iter()
                            .map(|row| match row.get("kind") {
                                "created" => crate::model::shared::event::ThreadEvent::Created(
                                    crate::model::shared::event::ThreadCreated {
                                        at: row.get("at"),
                                        content: row.get("content"),
                                        id: row.get("id"),
                                        thread_id: row.get("thread_id"),
                                        version: row.get("version"),
                                    },
                                ),
                                "replied" => crate::model::shared::event::ThreadEvent::Replied(
                                    crate::model::shared::event::ThreadReplied {
                                        at: row.get("at"),
                                        content: row.get("content"),
                                        id: row.get("id"),
                                        thread_id: row.get("thread_id"),
                                        version: row.get("version"),
                                    },
                                ),
                                _ => unreachable!(
                                    "Unknown event kind: {}",
                                    row.get::<String, _>("kind")
                                ),
                            })
                            .collect::<Vec<crate::model::shared::event::ThreadEvent>>()
                    });
                Ok(Some(crate::model::write::Thread::replay(&events?)))
            }
        }
    }

    async fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        if events.is_empty() {
            return Ok(());
        }

        let last_event = events.last().expect("events should not be empty");
        let thread_id = last_event.thread_id();
        let last_event_version = last_event.version();
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(SqliteStoreError::StoreBeginTransaction)?;
        match version {
            None => {
                sqlx::query(include_str!("sqlite_store/insert_thread_event_streams.sql"))
                    .bind(thread_id.to_string())
                    .bind(u32::from(last_event_version))
                    .execute(&mut *tx)
                    .await
                    .map_err(SqliteStoreError::StoreInsertEventStreams)?;
            }
            Some(_version) => {
                todo!()
            }
        }

        for event in events {
            let (at, content, id, kind, thread_id, version) = match event {
                crate::model::shared::event::ThreadEvent::Created(event) => (
                    event.at.clone(),
                    event.content.clone(),
                    event.id.clone(),
                    "created",
                    event.thread_id.clone(),
                    event.version,
                ),
                crate::model::shared::event::ThreadEvent::Replied(event) => (
                    event.at.clone(),
                    event.content.clone(),
                    event.id.clone(),
                    "replied",
                    event.thread_id.clone(),
                    event.version,
                ),
            };
            sqlx::query(include_str!("sqlite_store/insert_thread_events.sql"))
                .bind(at)
                .bind(content)
                .bind(id)
                .bind(kind)
                .bind(thread_id)
                .bind(u32::from(version))
                .execute(&mut *tx)
                .await
                .map_err(SqliteStoreError::StoreInsertEvents)?;
        }

        tx.commit().await.map_err(SqliteStoreError::StoreCommit)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::port::ThreadRepository;

    use super::*;

    #[tokio::test]
    async fn test_new() -> anyhow::Result<()> {
        let store = SqliteStore::new().await;

        let (created, created_events) =
            crate::model::write::Thread::create(crate::model::write::Message::new_for_testing())?;

        let found = store.find(created.id()).await?;
        assert!(found.is_none());

        store.store(None, &created_events).await?;

        let found = store.find(created.id()).await?;
        assert_eq!(found.map(|it| it.id().clone()), Some(created.id().clone()));
        Ok(())
    }
}
