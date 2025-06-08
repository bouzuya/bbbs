mod handler;
mod model;
mod utils;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    messages: Arc<Mutex<Vec<crate::model::read::Message>>>,
}

impl crate::handler::messages::MessageReader for AppState {
    fn get_message(
        &self,
        id: &crate::model::shared::id::MessageId,
    ) -> Option<crate::model::read::Message> {
        let messages = self.messages.lock().unwrap();
        let s = id.to_string();
        messages.iter().find(|it| &it.id == &s).cloned()
    }

    fn list_messages(&self) -> Vec<crate::model::read::Message> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }
}

impl crate::handler::messages::MessageRepository for AppState {
    fn store(
        &self,
        _version: Option<crate::model::write::Version>,
        message: &crate::model::write::Message,
    ) -> Result<(), handler::messages::MessageRepositoryError> {
        let mut messages = self.messages.lock().unwrap();
        messages.push(crate::model::read::Message {
            content: String::from(message.content.clone()),
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

    let router = handler::router().with_state(AppState {
        messages: Arc::new(Mutex::new(vec![])),
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap()
}
