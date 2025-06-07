use axum::extract::{Path, State};

use crate::handler::AskamaTemplateExt;
use crate::handler::messages::MessageReader;

#[derive(askama::Template)]
#[template(path = "messages/[id].html")]
pub struct MessageGetResponse {
    pub message: crate::read_model::Message,
}

impl AskamaTemplateExt for MessageGetResponse {}

impl axum::response::IntoResponse for MessageGetResponse {
    fn into_response(self) -> axum::response::Response {
        self.to_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageGetError {
    #[error("not found")]
    NotFound,
}

impl axum::response::IntoResponse for MessageGetError {
    fn into_response(self) -> axum::response::Response {
        match self {
            MessageGetError::NotFound => axum::http::StatusCode::NOT_FOUND.into_response(),
        }
    }
}

pub async fn handler<S: MessageReader>(
    State(state): State<S>,
    Path((id,)): Path<(String,)>,
) -> Result<MessageGetResponse, MessageGetError> {
    // TODO: validation
    let id = crate::read_model::MessageId(id);
    state
        .get_message(&id)
        .map(|message| MessageGetResponse { message })
        .ok_or_else(|| MessageGetError::NotFound)
}
