use std::str::FromStr as _;

use axum::extract::Path;
use axum::extract::{Form, State};

use crate::port::ThreadRepository;
use crate::port::ThreadRepositoryError;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ThreadReplyRequestBody {
    pub content: String,
    pub version: u32,
}

#[derive(serde::Serialize)]
pub struct ThreadReplyResponseBody {
    pub id: String,
}

impl axum::response::IntoResponse for ThreadReplyResponseBody {
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
pub enum ThreadReplyError {
    #[error("find")]
    Find(#[source] ThreadRepositoryError),
    #[error("invalid message content")]
    InvalidMessageContent(#[source] crate::model::write::MessageContentError),
    #[error("invalid thread id")]
    InvalidThreadId(#[source] crate::model::shared::id::ThreadIdError),
    #[error("not found {0:?}")]
    NotFound(crate::model::shared::id::ThreadId),
    #[error("reply")]
    Reply(#[source] crate::model::write::ThreadError),
    #[error("store")]
    Store(#[source] ThreadRepositoryError),
}

impl axum::response::IntoResponse for ThreadReplyError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ThreadReplyError::Find(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
            ThreadReplyError::InvalidMessageContent(_) => {
                axum::http::StatusCode::BAD_REQUEST.into_response()
            }
            ThreadReplyError::InvalidThreadId(_) => {
                axum::http::StatusCode::BAD_REQUEST.into_response()
            }
            ThreadReplyError::NotFound(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
            ThreadReplyError::Reply(_) => axum::http::StatusCode::BAD_REQUEST.into_response(),
            ThreadReplyError::Store(e) => match e {
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
    Path((thread_id,)): Path<(String,)>,
    State(state): State<S>,
    Form(ThreadReplyRequestBody { content, version }): Form<ThreadReplyRequestBody>,
) -> Result<ThreadReplyResponseBody, ThreadReplyError> {
    let content = crate::model::write::MessageContent::try_from(content)
        .map_err(ThreadReplyError::InvalidMessageContent)?;
    let thread_id = crate::model::shared::id::ThreadId::from_str(&thread_id)
        .map_err(ThreadReplyError::InvalidThreadId)?;
    let version = crate::model::write::Version::from(version);
    let message = crate::model::write::Message::create(content);

    let thread = ThreadRepository::find(&state, &thread_id)
        .map_err(ThreadReplyError::Find)?
        .ok_or_else(|| ThreadReplyError::NotFound(thread_id))?;
    let (_, events) = thread
        .reply(message.clone())
        .map_err(ThreadReplyError::Reply)?;
    ThreadRepository::store(&state, Some(version), &events).map_err(ThreadReplyError::Store)?;

    Ok(ThreadReplyResponseBody {
        id: thread.id().to_string(),
    })
}
