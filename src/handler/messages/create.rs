use axum::extract::{Form, State};

use crate::handler::messages::{MessageRepository, MessageRepositoryError};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MessageCreateRequestBody {
    pub content: String,
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
    #[error("repository error")]
    Repository(#[from] MessageRepositoryError),
}

impl axum::response::IntoResponse for MessageCreateError {
    fn into_response(self) -> axum::response::Response {
        match self {
            MessageCreateError::Repository(e) => match e {
                MessageRepositoryError::InternalError(_) => {
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                MessageRepositoryError::NotFound(_) => {
                    axum::http::StatusCode::NOT_FOUND.into_response()
                }
                MessageRepositoryError::VersionMismatch { .. } => {
                    axum::http::StatusCode::CONFLICT.into_response()
                }
            },
        }
    }
}

pub async fn handler<S: MessageRepository>(
    State(state): State<S>,
    Form(MessageCreateRequestBody { content }): Form<MessageCreateRequestBody>,
) -> Result<MessageCreateResponseBody, MessageCreateError> {
    let message = crate::write_model::Message::create(content);
    MessageRepository::store(&state, None, &message)?;
    Ok(MessageCreateResponseBody {
        id: message.id.to_string(),
    })
}
