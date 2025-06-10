mod handler;
mod model;
mod utils;

use std::sync::{Arc, Mutex};

use crate::model::shared::event::ThreadReplied;

struct Store {
    read: crate::model::read::Thread,
    write: Vec<crate::model::shared::event::ThreadEvent>,
}

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<Store>>,
}

impl AppState {
    fn new() -> Self {
        let events = vec![crate::model::shared::event::ThreadEvent::Created(
            crate::model::shared::event::ThreadCreated {
                at: crate::utils::date_time::DateTime::now().to_string(),
                content: "_".to_owned(),
                id: crate::model::shared::id::EventId::generate().to_string(),
                message_id: crate::model::shared::id::MessageId::generate().to_string(),
                thread_id: "5868d08d-12d7-468f-b77e-cb6e837baaf9".to_owned(),
                version: 1,
            },
        )];
        AppState {
            store: Arc::new(Mutex::new(Store {
                read: crate::model::read::Thread::replay(events.clone()),
                write: events,
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
        let s = id.to_string();
        store.read.messages.iter().find(|it| &it.id == &s).cloned()
    }

    fn list_messages(&self) -> Vec<crate::model::read::Message> {
        let store = self.store.lock().unwrap();
        store.read.messages.clone()
    }
}

impl crate::handler::messages::ThreadRepository for AppState {
    fn find(
        &self,
        _id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, handler::messages::MessageRepositoryError>
    {
        // TODO: use id
        let store = self.store.lock().unwrap();
        Ok(Some(crate::model::write::Thread::replay(&store.write)))
    }

    fn store(
        &self,
        _version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), handler::messages::MessageRepositoryError> {
        let mut store = self.store.lock().unwrap();
        // TODO: check version
        store.write.extend_from_slice(events);
        for event in events {
            match event {
                model::shared::event::ThreadEvent::Created(_) => todo!(),
                model::shared::event::ThreadEvent::Replied(ThreadReplied {
                    at,
                    content,
                    id: _,
                    message_id,
                    thread_id: _,
                    version: _,
                }) => {
                    store.read.messages.push(crate::model::read::Message {
                        content: content.to_owned(),
                        created_at: at.to_owned(),
                        id: message_id.to_owned(),
                    });
                }
            }
        }
        Ok(())
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
