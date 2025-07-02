use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::model::read::ThreadWithoutMessages;

struct InMemoryStoreInner {
    read: BTreeMap<crate::model::shared::id::ThreadId, crate::model::read::Thread>,
    write:
        BTreeMap<crate::model::shared::id::ThreadId, Vec<crate::model::shared::event::ThreadEvent>>,
}

#[derive(Clone)]
pub struct InMemoryStore(Arc<Mutex<InMemoryStoreInner>>);

impl InMemoryStore {
    pub fn new() -> Self {
        InMemoryStore(Arc::new(Mutex::new(InMemoryStoreInner {
            read: BTreeMap::new(),
            write: BTreeMap::new(),
        })))
    }
}

impl crate::store::Store for InMemoryStore {}

#[async_trait::async_trait]
impl crate::port::ThreadReader for InMemoryStore {
    async fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::read::Thread>, crate::port::ThreadReaderError> {
        let store = self.0.lock().unwrap();
        Ok(store.read.get(id).cloned())
    }

    async fn list_threads(
        &self,
    ) -> Result<Vec<crate::model::read::ThreadWithoutMessages>, crate::port::ThreadReaderError>
    {
        let store = self.0.lock().unwrap();
        Ok(store
            .read
            .values()
            .cloned()
            .map(ThreadWithoutMessages::from)
            .collect())
    }
}

#[async_trait::async_trait]
impl crate::port::ThreadRepository for InMemoryStore {
    async fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        let store = self.0.lock().unwrap();
        Ok(store
            .write
            .get(id)
            .map(|events| crate::model::write::Thread::replay(events)))
    }

    async fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        let mut store = self.0.lock().unwrap();
        if events.is_empty() {
            return Ok(());
        }
        let thread_id = events[0].thread_id();

        match version {
            None => match store.write.get_mut(&thread_id) {
                Some(stored_events) => {
                    let stored_version = stored_events
                        .last()
                        .map(|last_event| last_event.version())
                        .expect("stored_events not to be empty");
                    return Err(crate::port::ThreadRepositoryError::VersionMismatch {
                        actual: stored_version,
                        expected: version,
                    });
                }
                None => {
                    store.write.insert(thread_id.clone(), events.to_vec());
                }
            },
            Some(version) => match store.write.get_mut(&thread_id) {
                Some(stored_events) => {
                    let stored_version = stored_events
                        .last()
                        .map(|last_event| last_event.version())
                        .expect("stored_events not to be empty");
                    if stored_version != version {
                        return Err(crate::port::ThreadRepositoryError::VersionMismatch {
                            actual: stored_version,
                            expected: Some(version),
                        });
                    }
                    stored_events.extend_from_slice(events);
                }
                None => return Err(crate::port::ThreadRepositoryError::NotFound(thread_id)),
            },
        }

        match store.read.get_mut(&thread_id) {
            Some(thread) => {
                for event in events {
                    thread.apply(event.clone());
                }
            }
            None => {
                let thread = crate::model::read::Thread::replay(events.to_vec());
                store.read.insert(thread_id, thread);
            }
        }

        Ok(())
    }
}
