use axum::extract::State;

use crate::handler::AskamaTemplateExt;
use crate::handler::messages::MessageReader;

#[derive(askama::Template)]
#[template(path = "messages/index.html")]
pub struct MessageListResponse {
    pub messages: Vec<String>,
}

impl AskamaTemplateExt for MessageListResponse {}

impl axum::response::IntoResponse for MessageListResponse {
    fn into_response(self) -> axum::response::Response {
        self.to_response()
    }
}

pub async fn handle<S: MessageReader>(State(state): State<S>) -> MessageListResponse {
    MessageListResponse {
        messages: state
            .list_messages()
            .into_iter()
            .map(|it| it.content)
            .collect::<Vec<String>>(),
    }
}
