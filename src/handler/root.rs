use axum::extract::State;

pub trait Name {
    fn name(&self) -> &str;
}

async fn handler<S: Name>(State(state): State<S>) -> String {
    format!("Hello, {}!", state.name())
}

pub fn router<S: Clone + self::Name + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new().route("/", axum::routing::get(handler::<S>))
}

#[cfg(test)]
mod tests {
    use crate::handler::tests::ResponseExt;
    use crate::handler::tests::send_request;

    use super::*;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        #[derive(Clone)]
        struct AppState;
        impl Name for AppState {
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
        assert_eq!(response.into_body_string().await?, "Hello, bouzuya!");
        Ok(())
    }
}
