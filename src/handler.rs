use axum::extract::State;

pub trait Name {
    fn name(&self) -> &str;
}

async fn root<S: Name>(State(state): State<S>) -> String {
    format!("Hello, {}!", state.name())
}

pub fn router<S: Clone + Name + Send + Sync + 'static>() -> axum::Router<S> {
    axum::Router::new().route("/", axum::routing::get(root::<S>))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_root() -> anyhow::Result<()> {
        #[derive(Clone)]
        struct AppState;
        impl Name for AppState {
            fn name(&self) -> &str {
                "bouzuya"
            }
        }
        let router = router().with_state(AppState);

        let response = tower::ServiceExt::oneshot(
            router,
            axum::http::Request::builder()
                .method(axum::http::Method::GET)
                .uri("/")
                .body(axum::body::Body::empty())?,
        )
        .await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await?
                .to_vec(),
        )?;
        assert_eq!(body, "Hello, bouzuya!");
        Ok(())
    }
}
