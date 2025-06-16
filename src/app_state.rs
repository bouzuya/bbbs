use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

struct ReadStore {
    threads: BTreeMap<crate::model::shared::id::ThreadId, crate::model::read::Thread>,
}

struct Store {
    read: ReadStore,
    write:
        BTreeMap<crate::model::shared::id::ThreadId, Vec<crate::model::shared::event::ThreadEvent>>,
}

#[derive(Clone)]
pub struct AppState {
    store: Arc<Mutex<Store>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            store: Arc::new(Mutex::new(Store {
                read: ReadStore {
                    threads: BTreeMap::new(),
                },
                write: BTreeMap::new(),
            })),
        }
    }
}

impl crate::port::ThreadRepository for AppState {
    fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        let store = self.store.lock().unwrap();
        Ok(store
            .write
            .get(id)
            .map(|events| crate::model::write::Thread::replay(events)))
    }

    fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        let mut store = self.store.lock().unwrap();
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

        match store.read.threads.get_mut(&thread_id) {
            Some(thread) => {
                for event in events {
                    thread.apply(event.clone());
                }
            }
            None => {
                let thread = crate::model::read::Thread::replay(events.to_vec());
                store.read.threads.insert(thread_id, thread);
            }
        }

        Ok(())
    }
}

impl crate::port::ThreadReader for AppState {
    fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Option<crate::model::read::Thread> {
        let store = self.store.lock().unwrap();
        store.read.threads.get(id).cloned()
    }

    fn list_threads(&self) -> Vec<crate::model::read::Thread> {
        let store = self.store.lock().unwrap();
        store.read.threads.values().cloned().collect()
    }
}
