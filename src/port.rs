pub trait MessageReader {
    fn get_message(
        &self,
        id: &crate::model::shared::id::MessageId,
    ) -> Option<crate::model::read::Message>;
    fn list_messages(&self) -> Vec<crate::model::read::Message>;
}

pub trait ThreadReader {
    fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Option<crate::model::read::Thread>;

    fn list_threads(&self) -> Vec<crate::model::read::Thread>;
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

pub trait ThreadRepository {
    fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, ThreadRepositoryError>;

    fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), ThreadRepositoryError>;
}
