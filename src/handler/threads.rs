mod create;
mod get;
mod list;
mod reply;

pub fn router<
    S: Clone + crate::port::ThreadReader + crate::port::ThreadRepository + Send + Sync + 'static,
>() -> axum::Router<S> {
    axum::Router::new()
        .route(
            "/threads",
            axum::routing::get(self::list::handler::<S>).post(self::create::handler::<S>),
        )
        .route("/threads/{id}", axum::routing::get(self::get::handler::<S>))
        .route(
            "/threads/{id}/reply",
            axum::routing::post(self::reply::handler::<S>),
        )
}

#[cfg(test)]
mod tests {
    use crate::handler::tests::ResponseExt;
    use crate::handler::tests::send_request;

    use super::*;

    #[derive(Clone)]
    struct AppState(Vec<crate::model::read::Thread>);

    impl crate::port::ThreadReader for AppState {
        fn get_thread(
            &self,
            id: &crate::model::shared::id::ThreadId,
        ) -> Option<crate::model::read::Thread> {
            let s = id.to_string();
            self.0.iter().find(|it| &it.id == &s).cloned()
        }

        fn list_threads(&self) -> Vec<crate::model::read::Thread> {
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
            _events: &[crate::model::shared::event::ThreadEvent],
        ) -> Result<(), crate::port::ThreadRepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/threads/9b018a80-edcf-4a7b-89be-cc807bc2e647")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body_string().await?;
        assert!(body.contains("New thread content"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_not_found() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/threads/1df49bbd-3f94-475b-a057-d9d4c827449f")
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
            .uri("/threads")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = response.into_body_string().await?;
        assert!(body.contains("/threads/9b018a80-edcf-4a7b-89be-cc807bc2e647"));
        assert!(body.contains("New thread content"));
        assert!(body.contains("/threads/a2d3f8e9-4c5b-6d7e-8f9a-0b1c2d3e4f5g"));
        assert!(body.contains("Test Thread 2"));
        Ok(())
    }

    #[tokio::test]
    async fn test_create() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/threads")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(axum::body::Body::from("content=New thread content"))?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        Ok(())
    }

    #[tokio::test]
    async fn test_reply() -> anyhow::Result<()> {
        let router = router().with_state(build_app_state());

        let request = axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/threads/9b018a80-edcf-4a7b-89be-cc807bc2e647/reply")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(axum::body::Body::from("content=Reply content&version=1"))?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::SEE_OTHER);
        Ok(())
    }

    fn build_app_state() -> AppState {
        use crate::model::read::Thread;
        AppState(vec![
            Thread {
                id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
                messages: vec![
                    crate::model::read::Message {
                        content: "New thread content".to_owned(),
                        created_at: "2020-01-02T03:04:05Z".to_owned(),
                        id: "28cec994-e1c6-4987-b151-a4e66db42bda".to_owned(),
                        thread_id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
                    },
                    crate::model::read::Message {
                        content: "Reply content".to_owned(),
                        created_at: "2020-01-02T04:05:06Z".to_owned(),
                        id: "e77392c9-3883-456c-9add-288e4c2ca980".to_owned(),
                        thread_id: "9b018a80-edcf-4a7b-89be-cc807bc2e647".to_owned(),
                    },
                ],
                replies_count: 1,
                version: 2,
            },
            Thread {
                id: "a2d3f8e9-4c5b-6d7e-8f9a-0b1c2d3e4f5g".to_owned(),
                messages: vec![crate::model::read::Message {
                    content: "Test Thread 2".to_owned(),
                    created_at: "2020-01-02T05:06:07Z".to_owned(),
                    id: "7402f8d6-7f12-40f5-875d-b473ac7306c5".to_owned(),
                    thread_id: "a2d3f8e9-4c5b-6d7e-8f9a-0b1c2d3e4f5g".to_owned(),
                }],
                replies_count: 0,
                version: 1,
            },
        ])
    }
}
