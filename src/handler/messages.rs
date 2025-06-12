mod create;
mod get;
mod list;

pub fn router<
    S: Clone + crate::port::MessageReader + crate::port::ThreadRepository + Send + Sync + 'static,
>() -> axum::Router<S> {
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
    struct AppState(Vec<crate::model::read::Message>);
    impl crate::port::MessageReader for AppState {
        fn get_message(
            &self,
            id: &crate::model::shared::id::MessageId,
        ) -> Option<crate::model::read::Message> {
            let s = id.to_string();
            self.0.iter().find(|it| &it.id == &s).cloned()
        }

        fn list_messages(&self) -> Vec<crate::model::read::Message> {
            self.0.clone()
        }
    }
    impl crate::port::ThreadRepository for AppState {
        fn find(
            &self,
            _id: &crate::model::shared::id::ThreadId,
        ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError>
        {
            let (thread, _) = crate::model::write::Thread::create(
                crate::model::write::Message::new_for_testing(),
            )
            .expect("dummy thread creation to be successful");
            Ok(Some(thread))
        }

        fn store(
            &self,
            _version: Option<crate::model::write::Version>,
            _thread: &[crate::model::shared::event::ThreadEvent],
        ) -> Result<(), crate::port::ThreadRepositoryError> {
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
                    thread_id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
                    version: 1,
                },
            )?))?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/messages/28cec994-e1c6-4987-b151-a4e66db42bda")
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
            .uri("/messages/1df49bbd-3f94-475b-a057-d9d4c827449f")
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
        assert!(body.contains("/messages/28cec994-e1c6-4987-b151-a4e66db42bda"));
        assert!(body.contains("foo"));
        assert!(body.contains("/messages/e77392c9-3883-456c-9add-288e4c2ca980"));
        assert!(body.contains("bar"));
        assert!(body.contains("/messages/7402f8d6-7f12-40f5-875d-b473ac7306c5"));
        assert!(body.contains("baz"));
        Ok(())
    }

    fn build_app_state() -> AppState {
        use crate::model::read::Message;
        AppState(vec![
            Message {
                content: "foo".to_owned(),
                created_at: "2020-01-02T03:04:05Z".to_owned(),
                id: "28cec994-e1c6-4987-b151-a4e66db42bda".to_owned(),
                thread_id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
            },
            Message {
                content: "bar".to_owned(),
                created_at: "2020-01-03T03:04:05Z".to_owned(),
                id: "e77392c9-3883-456c-9add-288e4c2ca980".to_owned(),
                thread_id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
            },
            Message {
                content: "baz".to_owned(),
                created_at: "2020-01-04T03:04:05Z".to_owned(),
                id: "7402f8d6-7f12-40f5-875d-b473ac7306c5".to_owned(),
                thread_id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
            },
        ])
    }
}
