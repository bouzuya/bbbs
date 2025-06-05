mod handler;
mod read_model;

#[derive(Clone)]
struct AppState {
    messages: Vec<crate::read_model::Message>,
    name: String,
}

impl crate::handler::message::MessageReader for AppState {
    fn get_message(&self, id: &str) -> Option<crate::read_model::Message> {
        self.messages.iter().find(|it| it.id == id).cloned()
    }

    fn list_messages(&self) -> Vec<crate::read_model::Message> {
        self.messages.clone()
    }
}

impl crate::handler::root::Name for AppState {
    fn name(&self) -> &str {
        &self.name
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
    let router = handler::router().with_state(AppState {
        messages: vec![
            Message {
                content: "foo".to_owned(),
                id: "1".to_owned(),
            },
            Message {
                content: "bar".to_owned(),
                id: "2".to_owned(),
            },
            Message {
                content: "baz".to_owned(),
                id: "3".to_owned(),
            },
        ],
        name: "bouzuya".to_owned(),
    });
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap()
}
