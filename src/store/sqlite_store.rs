use sqlx::Row as _;

#[derive(Clone)]
pub struct SqliteStore(sqlx::SqlitePool);

impl SqliteStore {
    pub async fn new() -> Self {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite:./bbbs.sqlite?mode=rwc")
            .await
            .unwrap();

        // write
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

        // read
        sqlx::query(
            r#"
CREATE TABLE IF NOT EXISTS threads (
    created_at              TEXT    NOT NULL,
    id                      TEXT    NOT NULL   PRIMARY KEY,
    last_message_content    TEXT    NOT NULL,
    last_message_created_at TEXT    NOT NULL,
    last_message_number     INTEGER NOT NULL,
    replies_count           INTEGER NOT NULL,
    version                 INTEGER NOT NULL
)"#,
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
CREATE TABLE IF NOT EXISTS messages (
    content     TEXT    NOT NULL,
    created_at  TEXT    NOT NULL,
    thread_id   TEXT    NOT NULL,
    number      INTEGER NOT NULL,
    PRIMARY KEY (thread_id, number)
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
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::read::Thread>, crate::port::ThreadReaderError> {
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(SqliteStoreError::GetThreadBeginTransaction)?;
        let row = sqlx::query(include_str!("sqlite_store/select_threads.sql"))
            .bind(id.to_string())
            .fetch_optional(&mut *tx)
            .await
            .map_err(SqliteStoreError::GetThreadSelectThread)?;
        let thread = match row {
            None => None,
            Some(row) => Some(crate::model::read::Thread {
                id: row.get("id"),
                created_at: row.get("created_at"),
                last_message: crate::model::read::Message {
                    content: row.get("last_message_content"),
                    created_at: row.get("last_message_created_at"),
                    number: row.get("last_message_number"),
                },
                // FIXME
                messages: vec![],
                replies_count: row.get("replies_count"),
                version: row.get("version"),
            }),
        };
        tx.rollback()
            .await
            .map_err(SqliteStoreError::GetThreadRollback)?;
        Ok(thread)
    }

    async fn list_threads(
        &self,
    ) -> Result<Vec<crate::model::read::Thread>, crate::port::ThreadReaderError> {
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(SqliteStoreError::ListThreadsBeginTransaction)?;
        let rows = sqlx::query(include_str!("sqlite_store/select_threads_all.sql"))
            .fetch_all(&mut *tx)
            .await
            .map_err(SqliteStoreError::ListThreadsSelectThread)?;
        let threads = rows
            .into_iter()
            .map(|row| crate::model::read::Thread {
                id: row.get("id"),
                created_at: row.get("created_at"),
                last_message: crate::model::read::Message {
                    content: row.get("last_message_content"),
                    created_at: row.get("last_message_created_at"),
                    number: row.get("last_message_number"),
                },
                // FIXME
                messages: vec![],
                replies_count: row.get("replies_count"),
                version: row.get("version"),
            })
            .collect::<Vec<crate::model::read::Thread>>();
        tx.rollback()
            .await
            .map_err(SqliteStoreError::ListThreadsRollback)?;
        Ok(threads)
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
    #[error("get thread begin transaction")]
    GetThreadBeginTransaction(#[source] sqlx::Error),
    #[error("get thread rollback")]
    GetThreadRollback(#[source] sqlx::Error),
    #[error("get thread select thread")]
    GetThreadSelectThread(#[source] sqlx::Error),
    #[error("list threads begin transaction")]
    ListThreadsBeginTransaction(#[source] sqlx::Error),
    #[error("list threads select thread")]
    ListThreadsSelectThread(#[source] sqlx::Error),
    #[error("list threads rollback")]
    ListThreadsRollback(#[source] sqlx::Error),
    #[error("store begin transaction")]
    StoreBeginTransaction(#[source] sqlx::Error),
    #[error("store commit")]
    StoreCommit(#[source] sqlx::Error),
    #[error("store insert event streams")]
    StoreInsertEventStreams(#[source] sqlx::Error),
    #[error("store update event streams")]
    StoreUpdateEventStreams(#[source] sqlx::Error),
    #[error(
        "store update event streams conflict (expected version: {expected_version:?}, thread id: {thread_id})"
    )]
    StoreUpdateEventStreamsConflict {
        expected_version: crate::model::write::Version,
        thread_id: crate::model::shared::id::ThreadId,
    },
    #[error("store update read model insert threads")]
    StoreUpdateReadModelInsertThreads(#[source] sqlx::Error),
    #[error("store update read model update threads")]
    StoreUpdateReadModelUpdateThreads(#[source] sqlx::Error),
    #[error("store insert events")]
    StoreInsertEvents(#[source] sqlx::Error),
}

impl From<SqliteStoreError> for crate::port::ThreadReaderError {
    fn from(err: SqliteStoreError) -> Self {
        Self(err.into())
    }
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
        let mut tx = self
            .0
            .begin()
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
            Some(version) => {
                let result =
                    sqlx::query(include_str!("sqlite_store/update_thread_event_streams.sql"))
                        .bind(u32::from(last_event_version))
                        .bind(thread_id.to_string())
                        .bind(u32::from(version))
                        .execute(&mut *tx)
                        .await
                        .map_err(SqliteStoreError::StoreUpdateEventStreams)?;
                if result.rows_affected() == 0 {
                    return Err(SqliteStoreError::StoreUpdateEventStreamsConflict {
                        expected_version: version,
                        thread_id,
                    })?;
                }
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

        // Update read model
        for event in events {
            match event {
                crate::model::shared::event::ThreadEvent::Created(event) => {
                    sqlx::query(include_str!("sqlite_store/insert_threads.sql"))
                        .bind(event.at.clone())
                        .bind(event.thread_id.clone())
                        .bind(event.content.clone())
                        .bind(event.at.clone())
                        .bind(1_i64)
                        .bind(0_i64)
                        .bind(event.version)
                        .execute(&mut *tx)
                        .await
                        .map_err(SqliteStoreError::StoreUpdateReadModelInsertThreads)?;
                }
                crate::model::shared::event::ThreadEvent::Replied(event) => {
                    sqlx::query(include_str!("sqlite_store/update_threads.sql"))
                        .bind(event.content.clone())
                        .bind(event.at.clone())
                        .bind(event.version)
                        .bind(event.thread_id.clone())
                        .bind(event.version - 1)
                        .execute(&mut *tx)
                        .await
                        .map_err(SqliteStoreError::StoreUpdateReadModelUpdateThreads)?;
                }
            }
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
        assert_eq!(found, Some(created.clone()));

        let (replied, replied_events) =
            created.reply(crate::model::write::Message::new_for_testing())?;
        store
            .store(Some(created.version()), &replied_events)
            .await?;

        let found = store.find(replied.id()).await?;
        assert_eq!(found, Some(replied));

        Ok(())
    }
}
