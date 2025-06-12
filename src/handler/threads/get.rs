use std::str::FromStr as _;

use axum::extract::{Path, State};

use crate::handler::AskamaTemplateExt;
use crate::port::ThreadReader;

#[derive(askama::Template)]
#[template(path = "threads/[id].html")]
pub struct ThreadGetResponse {
    pub thread: crate::model::read::Thread,
}

impl AskamaTemplateExt for ThreadGetResponse {}

impl axum::response::IntoResponse for ThreadGetResponse {
    fn into_response(self) -> axum::response::Response {
        self.to_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ThreadGetError {
    #[error("invalid thread id")]
    InvalidId(#[from] crate::model::shared::id::ThreadIdError),
    #[error("not found")]
    NotFound,
}

impl axum::response::IntoResponse for ThreadGetError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ThreadGetError::InvalidId(_) => axum::http::StatusCode::BAD_REQUEST.into_response(),
            ThreadGetError::NotFound => axum::http::StatusCode::NOT_FOUND.into_response(),
        }
    }
}

pub async fn handler<S: ThreadReader>(
    State(state): State<S>,
    Path((id,)): Path<(String,)>,
) -> Result<ThreadGetResponse, ThreadGetError> {
    let id =
        crate::model::shared::id::ThreadId::from_str(&id).map_err(ThreadGetError::InvalidId)?;
    state
        .get_thread(&id)
        .map(|thread| ThreadGetResponse { thread })
        .ok_or_else(|| ThreadGetError::NotFound)
}
