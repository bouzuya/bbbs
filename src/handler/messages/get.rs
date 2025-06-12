use std::str::FromStr as _;

use axum::extract::{Path, State};

use crate::handler::AskamaTemplateExt;
use crate::port::MessageReader;

#[derive(askama::Template)]
#[template(path = "messages/[id].html")]
pub struct MessageGetResponse {
    pub message: crate::model::read::Message,
}

impl AskamaTemplateExt for MessageGetResponse {}

impl axum::response::IntoResponse for MessageGetResponse {
    fn into_response(self) -> axum::response::Response {
        self.to_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageGetError {
    #[error("invalid message id")]
    InvalidId(#[from] crate::model::shared::id::MessageIdError),
    #[error("not found")]
    NotFound,
}

impl axum::response::IntoResponse for MessageGetError {
    fn into_response(self) -> axum::response::Response {
        match self {
            MessageGetError::InvalidId(_) => axum::http::StatusCode::BAD_REQUEST.into_response(),
            MessageGetError::NotFound => axum::http::StatusCode::NOT_FOUND.into_response(),
        }
    }
}

pub async fn handler<S: MessageReader>(
    State(state): State<S>,
    Path((id,)): Path<(String,)>,
) -> Result<MessageGetResponse, MessageGetError> {
    let id =
        crate::model::shared::id::MessageId::from_str(&id).map_err(MessageGetError::InvalidId)?;
    state
        .get_message(&id)
        .map(|message| MessageGetResponse { message })
        .ok_or_else(|| MessageGetError::NotFound)
}
