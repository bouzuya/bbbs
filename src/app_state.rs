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

impl crate::port::ThreadRepository for AppState {
    fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        self.store.find(id)
    }

    fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        self.store.store(version, events)
    }
}

impl crate::port::ThreadReader for AppState {
    fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Option<crate::model::read::Thread> {
        self.store.get_thread(id)
    }

    fn list_threads(&self) -> Vec<crate::model::read::Thread> {
        self.store.list_threads()
    }
}
