use std::sync::Arc;

#[cfg(not(feature = "sqlite"))]
use crate::store::InMemoryStore;
#[cfg(feature = "sqlite")]
use crate::store::SqliteStore;
use crate::store::Store;

#[derive(Clone)]
pub struct AppState {
    store: Arc<dyn Store + Send + Sync>,
}

impl AppState {
    #[cfg(feature = "sqlite")]
    pub async fn new() -> Self {
        AppState {
            store: Arc::new(SqliteStore::new().await),
        }
    }

    #[cfg(not(feature = "sqlite"))]
    pub async fn new() -> Self {
        AppState {
            store: Arc::new(InMemoryStore::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::port::ThreadRepository for AppState {
    async fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        self.store.find(id).await
    }

    async fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        self.store.store(version, events).await
    }
}

#[async_trait::async_trait]
impl crate::port::ThreadReader for AppState {
    async fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::read::Thread>, crate::port::ThreadReaderError> {
        self.store.get_thread(id).await
    }

    async fn list_threads(
        &self,
    ) -> Result<Vec<crate::model::read::Thread>, crate::port::ThreadReaderError> {
        self.store.list_threads().await
    }
}
