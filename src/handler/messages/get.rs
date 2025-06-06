use axum::extract::{Path, State};

use crate::handler::messages::MessageReader;

pub async fn handle<S: MessageReader>(
    State(state): State<S>,
    Path((id,)): Path<(String,)>,
) -> (axum::http::StatusCode, String) {
    // TODO: validation
    let id = crate::read_model::MessageId(id);
    state
        .get_message(&id)
        .map(|message| message.content)
        .map(|content| (axum::http::StatusCode::OK, content.to_owned()))
        .unwrap_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                "Message not found".to_owned(),
            )
        })
}
