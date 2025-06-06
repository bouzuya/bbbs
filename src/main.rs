use std::sync::{Arc, Mutex};

mod handler;
mod read_model;
mod write_model;

#[derive(Clone)]
struct AppState {
    messages: Arc<Mutex<Vec<crate::read_model::Message>>>,
}

impl crate::handler::messages::MessageReader for AppState {
    fn get_message(&self, id: &crate::read_model::MessageId) -> Option<crate::read_model::Message> {
        let messages = self.messages.lock().unwrap();
        messages.iter().find(|it| &it.id == id).cloned()
    }

    fn list_messages(&self) -> Vec<crate::read_model::Message> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }
}

impl crate::handler::messages::MessageRepository for AppState {
    fn store(
        &self,
        _version: Option<crate::write_model::Version>,
        message: &crate::write_model::Message,
    ) -> Result<(), handler::messages::MessageRepositoryError> {
        let mut messages = self.messages.lock().unwrap();
        messages.push(crate::read_model::Message {
            content: message.content.clone(),
            id: crate::read_model::MessageId(message.id.to_string()),
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

    use crate::read_model::Message;
    use crate::read_model::MessageId;
    let router = handler::router().with_state(AppState {
        messages: Arc::new(Mutex::new(vec![
            Message {
                content: "foo".to_owned(),
                id: MessageId("1".to_owned()),
            },
            Message {
                content: "bar".to_owned(),
                id: MessageId("2".to_owned()),
            },
            Message {
                content: "baz".to_owned(),
                id: MessageId("3".to_owned()),
            },
        ])),
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap()
}
