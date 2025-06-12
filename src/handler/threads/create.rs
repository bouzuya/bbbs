use axum::extract::{Form, State};

use crate::model::write::Thread;
use crate::port::ThreadRepository;
use crate::port::ThreadRepositoryError;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ThreadCreateRequestBody {
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct ThreadCreateResponseBody {
    pub id: String,
}

impl axum::response::IntoResponse for ThreadCreateResponseBody {
    fn into_response(self) -> axum::response::Response {
        let location = format!("/threads/{}", self.id);
        axum::response::Response::builder()
            .status(axum::http::StatusCode::SEE_OTHER)
            .header(
                axum::http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .header(axum::http::header::LOCATION, location)
            .body(axum::body::Body::empty())
            .expect("failed to build response")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageCreateError {
    #[error("create")]
    Create(#[source] crate::model::write::ThreadError),
    #[error("invalid message content")]
    InvalidMessageContent(#[source] crate::model::write::MessageContentError),
    #[error("store")]
    Store(#[source] ThreadRepositoryError),
}

impl axum::response::IntoResponse for MessageCreateError {
    fn into_response(self) -> axum::response::Response {
        match self {
            MessageCreateError::Create(_) => axum::http::StatusCode::BAD_REQUEST.into_response(),
            MessageCreateError::InvalidMessageContent(_) => {
                axum::http::StatusCode::BAD_REQUEST.into_response()
            }
            MessageCreateError::Store(e) => match e {
                ThreadRepositoryError::InternalError(_) => {
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                ThreadRepositoryError::NotFound(_) => {
                    axum::http::StatusCode::NOT_FOUND.into_response()
                }
                ThreadRepositoryError::VersionMismatch { .. } => {
                    axum::http::StatusCode::CONFLICT.into_response()
                }
            },
        }
    }
}

pub async fn handler<S: ThreadRepository>(
    State(state): State<S>,
    Form(ThreadCreateRequestBody { content }): Form<ThreadCreateRequestBody>,
) -> Result<ThreadCreateResponseBody, MessageCreateError> {
    let content = crate::model::write::MessageContent::try_from(content)
        .map_err(MessageCreateError::InvalidMessageContent)?;
    let message = crate::model::write::Message::create(content);

    let (thread, events) = Thread::create(message.clone()).map_err(MessageCreateError::Create)?;
    ThreadRepository::store(&state, None, &events).map_err(MessageCreateError::Store)?;

    Ok(ThreadCreateResponseBody {
        id: thread.id().to_string(),
    })
}
