use axum::extract::{Path, State};

pub trait MessageReader {
    fn get_message(&self, id: &crate::read_model::MessageId) -> Option<crate::read_model::Message>;
    fn list_messages(&self) -> Vec<crate::read_model::Message>;
}

async fn get<S: MessageReader>(
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

async fn list<S: MessageReader>(State(state): State<S>) -> String {
    state
        .list_messages()
        .into_iter()
        .map(|it| it.content)
        .collect::<Vec<String>>()
        .join(", ")
}

pub fn router<S: Clone + self::MessageReader + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new()
        .route("/messages", axum::routing::get(list::<S>))
        .route("/messages/{id}", axum::routing::get(get::<S>))
}

#[cfg(test)]
mod tests {
    use crate::handler::tests::ResponseExt;
    use crate::handler::tests::send_request;

    use super::*;

    #[derive(Clone)]
    struct AppState(Vec<crate::read_model::Message>);
    impl MessageReader for AppState {
        fn get_message(
            &self,
            id: &crate::read_model::MessageId,
        ) -> Option<crate::read_model::Message> {
            self.0.iter().find(|it| &it.id == id).cloned()
        }

        fn list_messages(&self) -> Vec<crate::read_model::Message> {
            self.0.clone()
        }
    }

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        use crate::read_model::Message;
        use crate::read_model::MessageId;
        let router = router().with_state(AppState(vec![
            Message {
                content: "foo".to_owned(),
                id: MessageId("1".to_owned()),
            },
            Message {
                content: "bar".to_owned(),
                id: MessageId("2".to_owned()),
            },
            Message {
                content: "baz".to_owned(),
                id: MessageId("3".to_owned()),
            },
        ]));

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages/1")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(response.into_body_string().await?, "foo");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_not_found() -> anyhow::Result<()> {
        use crate::read_model::Message;
        use crate::read_model::MessageId;
        let router = router().with_state(AppState(vec![
            Message {
                content: "foo".to_owned(),
                id: MessageId("1".to_owned()),
            },
            Message {
                content: "bar".to_owned(),
                id: MessageId("2".to_owned()),
            },
            Message {
                content: "baz".to_owned(),
                id: MessageId("3".to_owned()),
            },
        ]));

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages/4")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
        assert_eq!(response.into_body_string().await?, "Message not found");
        Ok(())
    }

    #[tokio::test]
    async fn test_list() -> anyhow::Result<()> {
        use crate::read_model::Message;
        use crate::read_model::MessageId;
        let router = router().with_state(AppState(vec![
            Message {
                content: "foo".to_owned(),
                id: MessageId("1".to_owned()),
            },
            Message {
                content: "bar".to_owned(),
                id: MessageId("2".to_owned()),
            },
            Message {
                content: "baz".to_owned(),
                id: MessageId("3".to_owned()),
            },
        ]));

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(response.into_body_string().await?, "foo, bar, baz");
        Ok(())
    }
}
