use axum::extract::State;

use crate::handler::AskamaTemplateExt;
use crate::handler::messages::MessageReader;

#[derive(askama::Template)]
#[template(path = "messages/index.html")]
pub struct MessageListResponse {
    pub thread_id: String,
    pub messages: Vec<crate::model::read::Message>,
}

impl AskamaTemplateExt for MessageListResponse {}

impl axum::response::IntoResponse for MessageListResponse {
    fn into_response(self) -> axum::response::Response {
        self.to_response()
    }
}

pub async fn handler<S: MessageReader>(State(state): State<S>) -> MessageListResponse {
    MessageListResponse {
        // TODO: dummy thread id
        thread_id: "5868d08d-12d7-468f-b77e-cb6e837baaf9".to_owned(),
        messages: state.list_messages(),
    }
}
