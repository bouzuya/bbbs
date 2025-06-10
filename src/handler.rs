pub mod messages;
pub mod root;

pub fn router<
    S: Clone
        + self::messages::MessageReader
        + self::messages::ThreadRepository
        + Send
        + Sync
        + 'static,
>() -> axum::Router<S> {
    axum::Router::new()
        .merge(self::messages::router::<S>())
        .merge(self::root::router::<S>())
}

trait AskamaTemplateExt: askama::Template {
    fn to_response(&self) -> axum::response::Response {
        askama::Template::render(&self)
            .map(|it| {
                axum::response::Response::builder()
                    .status(self.status_code())
                    .header(axum::http::header::CONTENT_TYPE, "text/html")
                    .body(axum::body::Body::new(it))
                    .expect("failed to build response")
            })
            .unwrap_or_else(|_e| {
                // TODO: tracing::error!(e);
                axum::response::IntoResponse::into_response(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
            })
    }

    fn status_code(&self) -> axum::http::StatusCode {
        axum::http::StatusCode::OK
    }
}

#[cfg(test)]
mod tests {
    use axum::response::Response;

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
}
