pub mod root;

pub fn router<S: Clone + self::root::Name + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new().merge(self::root::router::<S>())
}

#[cfg(test)]
mod tests {
    use axum::response::Response;

    use super::*;

    pub(crate) async fn send_request(
        router: axum::Router<()>,
        request: axum::http::Request<axum::body::Body>,
    ) -> anyhow::Result<Response> {
        let response = tower::ServiceExt::oneshot(router, request).await?;
        Ok(response)
    }

    pub(crate) trait ResponseExt {
        async fn into_body_string(self) -> anyhow::Result<String>;
    }

    impl ResponseExt for axum::response::Response<axum::body::Body> {
        async fn into_body_string(self) -> anyhow::Result<String> {
            let bytes = axum::body::to_bytes(self.into_body(), usize::MAX).await?;
            Ok(String::from_utf8(bytes.to_vec())?)
        }
    }

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        #[derive(Clone)]
        struct AppState;
        impl self::root::Name for AppState {
            fn name(&self) -> &str {
                "bouzuya"
            }
        }
        let router = router().with_state(AppState);

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        Ok(())
    }
}
