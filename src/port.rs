#[derive(Debug, thiserror::Error)]
#[error("thread reader error")]
pub struct ThreadReaderError(#[source] pub Box<dyn std::error::Error + Send + Sync>);

#[async_trait::async_trait]
pub trait ThreadReader {
    async fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::read::Thread>, ThreadReaderError>;

    async fn list_threads(
        &self,
    ) -> Result<Vec<crate::model::read::ThreadWithoutMessages>, ThreadReaderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ThreadRepositoryError {
    #[error("internal error: {0}")]
    InternalError(Box<dyn std::error::Error + Send + Sync>),

    #[error("not found {0:?}")]
    NotFound(crate::model::shared::id::ThreadId),

    #[error("version mismatch (expected: {expected:?}, actual: {actual:?})")]
    VersionMismatch {
        actual: crate::model::write::Version,
        expected: Option<crate::model::write::Version>,
    },
}

#[async_trait::async_trait]
pub trait ThreadRepository {
    async fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, ThreadRepositoryError>;

    async fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), ThreadRepositoryError>;
}
