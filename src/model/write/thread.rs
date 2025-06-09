use crate::model::shared::id::ThreadId;
use crate::model::write::Message;
use crate::model::write::Version;

pub struct Thread {
    pub id: ThreadId,
    pub messages: Vec<Message>,
    pub version: Version,
}

impl Thread {
    pub fn create(message: Message) -> Self {
        Self {
            id: ThreadId::generate(),
            messages: vec![message],
            version: Version::initial(),
        }
    }

    pub fn id(&self) -> &ThreadId {
        &self.id
    }

    pub fn message(&self) -> &Message {
        self.messages
            .first()
            .expect("Thread to have at least one message")
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let message = Message::new_for_testing();
        let thread = Thread::create(message.clone());
        assert!(!thread.id().to_string().is_empty());
        assert_eq!(thread.message(), &message);
        assert_eq!(thread.version(), Version::initial());
    }
}
