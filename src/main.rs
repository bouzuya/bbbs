mod handler;
mod model;
mod utils;

use std::{
    collections::BTreeMap,
    str::FromStr as _,
    sync::{Arc, Mutex},
};

use crate::model::shared::event::{ThreadCreated, ThreadReplied};

struct ReadStore {
    messages: BTreeMap<crate::model::shared::id::MessageId, crate::model::read::Message>,
    threads: BTreeMap<crate::model::shared::id::ThreadId, crate::model::read::Thread>,
}

struct Store {
    read: ReadStore,
    write:
        BTreeMap<crate::model::shared::id::ThreadId, Vec<crate::model::shared::event::ThreadEvent>>,
}

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<Store>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            store: Arc::new(Mutex::new(Store {
                read: ReadStore {
                    messages: BTreeMap::new(),
                    threads: BTreeMap::new(),
                },
                write: BTreeMap::new(),
            })),
        }
    }
}

impl crate::handler::messages::MessageReader for AppState {
    fn get_message(
        &self,
        id: &crate::model::shared::id::MessageId,
    ) -> Option<crate::model::read::Message> {
        let store = self.store.lock().unwrap();
        store.read.messages.get(id).cloned()
    }

    fn list_messages(&self) -> Vec<crate::model::read::Message> {
        let store = self.store.lock().unwrap();
        store.read.messages.values().cloned().collect()
    }
}

impl crate::handler::messages::ThreadRepository for AppState {
    fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, handler::messages::ThreadRepositoryError> {
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
    ) -> Result<(), handler::messages::ThreadRepositoryError> {
        let mut store = self.store.lock().unwrap();
        if events.is_empty() {
            return Ok(());
        }
        let thread_id = events[0].thread_id();

        match version {
            None => match store.write.get_mut(&thread_id) {
                Some(_) => todo!("conflict"),
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
                        todo!("conflict");
                    }
                    stored_events.extend_from_slice(events);
                }
                None => todo!("not found"),
            },
        }

        for event in events {
            match event {
                model::shared::event::ThreadEvent::Created(ThreadCreated {
                    at,
                    content,
                    id: _,
                    message_id,
                    thread_id: _,
                    version: _,
                }) => {
                    let message_id = crate::model::shared::id::MessageId::from_str(message_id)
                        .expect("message_id to be valid");
                    let message = crate::model::read::Message {
                        id: message_id.to_string(),
                        content: content.clone(),
                        created_at: at.clone(),
                    };
                    store.read.messages.insert(message_id, message);
                }
                model::shared::event::ThreadEvent::Replied(ThreadReplied {
                    at,
                    content,
                    id: _,
                    message_id,
                    thread_id: _,
                    version: _,
                }) => {
                    let message_id = crate::model::shared::id::MessageId::from_str(message_id)
                        .expect("message_id to be valid");
                    let message = crate::model::read::Message {
                        id: message_id.to_string(),
                        content: content.clone(),
                        created_at: at.clone(),
                    };
                    store.read.messages.insert(message_id, message);
                }
            }
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

impl crate::handler::threads::ThreadReader for AppState {
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

#[derive(clap::Parser)]
struct Cli {
    #[clap(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let cli = <Cli as clap::Parser>::parse();
    let port = cli.port.unwrap_or(3000);

    let router = handler::router().with_state(AppState::new());
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap()
}
