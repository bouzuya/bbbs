mod create;
mod get;
mod list;

pub trait ThreadReader {
    fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Option<crate::model::read::Thread>;

    fn list_threads(&self) -> Vec<crate::model::read::Thread>;
}

pub fn router<
    S: Clone + self::ThreadReader + super::messages::ThreadRepository + Send + Sync + 'static,
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
