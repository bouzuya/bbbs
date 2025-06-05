pub mod message;
pub mod root;

pub fn router<
    S: Clone + self::root::Name + self::message::MessageReader + Send + Sync + 'static,
>() -> axum::Router<S> {
    axum::Router::new()
        .merge(self::root::router::<S>())
        .merge(self::message::router::<S>())
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
