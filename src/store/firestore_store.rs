// TODO
pub struct FirestoreStore;

#[async_trait::async_trait]
impl crate::port::ThreadReader for FirestoreStore {
    async fn get_thread(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::read::Thread>, crate::port::ThreadReaderError> {
        todo!()
    }

    async fn list_threads(
        &self,
    ) -> Result<Vec<crate::model::read::ThreadWithoutMessages>, crate::port::ThreadReaderError>
    {
        todo!()
    }
}

#[async_trait::async_trait]
impl crate::port::ThreadRepository for FirestoreStore {
    async fn find(
        &self,
        id: &crate::model::shared::id::ThreadId,
    ) -> Result<Option<crate::model::write::Thread>, crate::port::ThreadRepositoryError> {
        todo!()
    }

    async fn store(
        &self,
        version: Option<crate::model::write::Version>,
        events: &[crate::model::shared::event::ThreadEvent],
    ) -> Result<(), crate::port::ThreadRepositoryError> {
        todo!()
    }
}
