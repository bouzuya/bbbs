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

pub async fn handler<S: ThreadReader>(State(state): State<S>) -> ThreadListResponse {
    ThreadListResponse {
        threads: state.list_threads().await,
    }
}
