use std::str::FromStr as _;

use axum::extract::{Form, State};

use crate::port::ThreadRepository;
use crate::port::ThreadRepositoryError;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MessageCreateRequestBody {
    pub content: String,
    pub thread_id: String,
    pub version: u32,
}

#[derive(serde::Serialize)]
pub struct MessageCreateResponseBody {
    pub id: String,
}

impl axum::response::IntoResponse for MessageCreateResponseBody {
    fn into_response(self) -> axum::response::Response {
        let location = format!("/messages/{}", self.id);
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

impl axum::response::IntoResponse for MessageCreateError {
    fn into_response(self) -> axum::response::Response {
        match self {
            MessageCreateError::Find(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
            MessageCreateError::InvalidMessageContent(_) => {
                axum::http::StatusCode::BAD_REQUEST.into_response()
            }
            MessageCreateError::InvalidThreadId(_) => {
                axum::http::StatusCode::BAD_REQUEST.into_response()
            }
            MessageCreateError::NotFound(_) => axum::http::StatusCode::NOT_FOUND.into_response(),
            MessageCreateError::Reply(_) => axum::http::StatusCode::BAD_REQUEST.into_response(),
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
    Form(MessageCreateRequestBody {
        content,
        thread_id,
        version,
    }): Form<MessageCreateRequestBody>,
) -> Result<MessageCreateResponseBody, MessageCreateError> {
    let content = crate::model::write::MessageContent::try_from(content)
        .map_err(MessageCreateError::InvalidMessageContent)?;
    let thread_id = crate::model::shared::id::ThreadId::from_str(&thread_id)
        .map_err(MessageCreateError::InvalidThreadId)?;
    let version = crate::model::write::Version::from(version);
    let message = crate::model::write::Message::create(content);

    let thread = ThreadRepository::find(&state, &thread_id)
        .map_err(MessageCreateError::Find)?
        .ok_or_else(|| MessageCreateError::NotFound(thread_id))?;
    let (_, events) = thread
        .reply(message.clone())
        .map_err(MessageCreateError::Reply)?;
    ThreadRepository::store(&state, Some(version), &events).map_err(MessageCreateError::Store)?;

    Ok(MessageCreateResponseBody {
        id: message.id.to_string(),
    })
}
