use axum::extract::State;

use crate::handler::AskamaTemplateExt;
use crate::port::ThreadReader;

#[derive(askama::Template)]
#[template(path = "threads/index.html")]
pub struct ThreadListResponse {
    pub threads: Vec<crate::model::read::Thread>,
}

impl AskamaTemplateExt for ThreadListResponse {}

impl axum::response::IntoResponse for ThreadListResponse {
    fn into_response(self) -> axum::response::Response {
        self.to_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ThreadListError {
    #[error("find")]
    ListThreads(#[source] crate::port::ThreadReaderError),
}

impl axum::response::IntoResponse for ThreadListError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ThreadListError::ListThreads(_) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub async fn handler<S: ThreadReader>(
    State(state): State<S>,
) -> Result<ThreadListResponse, ThreadListError> {
    Ok(ThreadListResponse {
        threads: state
            .list_threads()
            .await
            .map_err(ThreadListError::ListThreads)?,
    })
}
