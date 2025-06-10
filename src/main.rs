mod handler;
mod model;
mod utils;

use std::sync::{Arc, Mutex};

struct Store {
    read: crate::model::read::Thread,
    write: crate::model::write::Thread,
}

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<Store>>,
}

impl AppState {
    fn new() -> Self {
        let (thread, events) =
            crate::model::write::Thread::create(crate::model::write::Message::create(
                crate::model::write::MessageContent::try_from("_".to_owned())
                    .expect("dummy message content to be valid"),
            ))
            .expect("dummy thread creation to be successful");
        AppState {
            store: Arc::new(Mutex::new(Store {
                read: crate::model::read::Thread::replay(events),
                write: thread,
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

impl crate::handler::messages::MessageRepository for AppState {
    fn store(
        &self,
        _version: Option<crate::model::write::Version>,
        // TODO: use event
        message: &crate::model::write::Message,
    ) -> Result<(), handler::messages::MessageRepositoryError> {
        let mut store = self.store.lock().unwrap();
        // TODO: add event
        store
            .write
            .reply(message.clone())
            .map_err(Into::into)
            .map_err(handler::messages::MessageRepositoryError::InternalError)?;
        // TODO: use apply
        store.read.messages.push(crate::model::read::Message {
            content: String::from(message.content.clone()),
            created_at: message.created_at.to_string(),
            id: message.id.to_string(),
        });
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
