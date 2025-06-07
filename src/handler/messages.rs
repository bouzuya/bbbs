mod create;
mod get;
mod list;

pub trait MessageReader {
    fn get_message(&self, id: &crate::read_model::MessageId) -> Option<crate::read_model::Message>;
    fn list_messages(&self) -> Vec<crate::read_model::Message>;
}

#[derive(Debug, thiserror::Error)]
pub enum MessageRepositoryError {
    #[error("internal error: {0}")]
    InternalError(Box<dyn std::error::Error + Send + Sync>),

    #[error("not found {0:?}")]
    NotFound(crate::write_model::MessageId),

    #[error("version mismatch (expected: {expected:?}, actual: {actual:?})")]
    VersionMismatch {
        actual: crate::write_model::Version,
        expected: crate::write_model::Version,
    },
}

pub trait MessageRepository {
    fn store(
        &self,
        version: Option<crate::write_model::Version>,
        message: &crate::write_model::Message,
    ) -> Result<(), MessageRepositoryError>;
}

pub fn router<S: Clone + self::MessageReader + self::MessageRepository + Send + Sync + 'static>()
-> axum::Router<S> {
    axum::Router::new()
        .route(
            "/messages",
            axum::routing::get(self::list::handler::<S>).post(self::create::handler::<S>),
        )
        .route(
            "/messages/{id}",
            axum::routing::get(self::get::handler::<S>),
        )
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
            self.0.iter().find(|it| &it.id == &id.0).cloned()
        }

        fn list_messages(&self) -> Vec<crate::read_model::Message> {
            self.0.clone()
        }
    }
    impl MessageRepository for AppState {
        fn store(
            &self,
            _version: Option<crate::write_model::Version>,
            _message: &crate::write_model::Message,
        ) -> Result<(), MessageRepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .header(
                axum::http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .method(axum::http::Method::POST)
            .uri("/messages")
            .body(axum::body::Body::new(serde_urlencoded::to_string(
                self::create::MessageCreateRequestBody {
                    content: "hello".to_owned(),
                },
            )?))?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::CREATED);
        assert!(response.into_body_string().await?.starts_with("id="));
        Ok(())
    }

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages/1")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body_string().await?;
        assert!(body.contains("foo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_not_found() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages/4")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_list() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body_string().await?;
        assert!(body.contains("/messages/1"));
        assert!(body.contains("foo"));
        assert!(body.contains("/messages/2"));
        assert!(body.contains("bar"));
        assert!(body.contains("/messages/3"));
        assert!(body.contains("baz"));
        Ok(())
    }

    fn build_app_state() -> AppState {
        use crate::read_model::Message;
        AppState(vec![
            Message {
                content: "foo".to_owned(),
                id: "1".to_owned(),
            },
            Message {
                content: "bar".to_owned(),
                id: "2".to_owned(),
            },
            Message {
                content: "baz".to_owned(),
                id: "3".to_owned(),
            },
        ])
    }
}
