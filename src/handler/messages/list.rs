use axum::extract::State;

use crate::handler::messages::MessageReader;

pub async fn handle<S: MessageReader>(State(state): State<S>) -> String {
    state
        .list_messages()
        .into_iter()
        .map(|it| it.content)
        .collect::<Vec<String>>()
        .join(", ")
}
