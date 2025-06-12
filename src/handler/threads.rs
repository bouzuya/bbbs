mod create;
mod get;
mod list;

pub fn router<
    S: Clone + crate::port::ThreadReader + crate::port::ThreadRepository + Send + Sync + 'static,
>() -> axum::Router<S> {
    axum::Router::new()
        .route(
            "/threads",
            axum::routing::get(self::list::handler::<S>).post(self::create::handler::<S>),
        )
        .route("/threads/{id}", axum::routing::get(self::get::handler::<S>))
}

#[cfg(test)]
mod tests {
    // TODO: tests
}
