use axum::extract::State;

#[derive(askama::Template)]
#[template(path = "root.html")]
pub struct RootResponse;

impl axum::response::IntoResponse for RootResponse {
    fn into_response(self) -> axum::response::Response {
        askama::Template::render(&self)
            .map(|it| {
                axum::response::Response::builder()
                    .status(axum::http::StatusCode::OK)
                    .header(axum::http::header::CONTENT_TYPE, "text/html")
                    .body(axum::body::Body::new(it))
                    .expect("failed to build response")
            })
            .unwrap_or_else(|_e| {
                // TODO: tracing::error!(e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }
}

async fn handler<S>(State(_): State<S>) -> RootResponse {
    RootResponse
}

pub fn router<S: Clone + Send + Sync + 'static>() -> axum::Router<S> {
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
        let router = router().with_state(AppState);

        let request = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/")
            .body(axum::body::Body::empty())?;
        let response = send_request(router, request).await?;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert!(response.into_body_string().await?.contains(&"bbbs"));
        Ok(())
    }
}
