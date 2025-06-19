use std::sync::Arc;

use crate::store::{InMemoryStore, Store};

#[derive(Clone)]
pub struct AppState {
    store: Arc<dyn Store + Send + Sync>,
}

impl AppState {
    pub fn new() -> Self {
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
    ) -> Option<crate::model::read::Thread> {
        self.store.get_thread(id).await
    }

    async fn list_threads(&self) -> Vec<crate::model::read::Thread> {
        self.store.list_threads().await
    }
}
