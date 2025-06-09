use crate::model::shared::id::ThreadId;
use crate::model::write::Message;
use crate::model::write::Version;

#[derive(Debug, thiserror::Error)]
#[error("thread error")]
pub struct ThreadError(#[source] Box<dyn std::error::Error + Send + Sync>);

pub struct Thread {
    pub id: ThreadId,
    pub messages: Vec<Message>,
    pub version: Version,
}

impl Thread {
    pub fn create(message: Message) -> Result<Self, ThreadError> {
        Ok(Self {
            id: ThreadId::generate(),
            messages: vec![message],
            version: Version::initial(),
        })
    }

    pub fn id(&self) -> &ThreadId {
        &self.id
    }

    pub fn message(&self) -> &Message {
        self.messages
            .first()
            .expect("Thread to have at least one message")
    }

    pub fn reply(&self, message: Message) -> Result<Self, ThreadError> {
        if self.messages.len() == 1000 {
            return Err(ThreadError(
                "Thread has reached the maximum number of messages".into(),
            ));
        }
        Ok(Self {
            id: self.id.clone(),
            messages: {
                let mut messages = self.messages.clone();
                messages.push(message);
                messages
            },
            version: self.version.next(),
        })
    }

    pub fn version(&self) -> Version {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() -> anyhow::Result<()> {
        let message = Message::new_for_testing();
        let thread = Thread::create(message.clone())?;
        assert!(!thread.id().to_string().is_empty());
        assert_eq!(thread.message(), &message);
        assert_eq!(thread.version(), Version::initial());
        Ok(())
    }

    #[test]
    fn test_reply() -> anyhow::Result<()> {
        let message = Message::new_for_testing();
        let thread = Thread::create(message.clone())?;
        let reply_message = Message::new_for_testing();
        let reply_thread = thread.reply(reply_message.clone())?;

        assert_eq!(reply_thread.id(), thread.id());
        assert_eq!(reply_thread.message(), &message);
        assert_eq!(reply_thread.version(), thread.version().next());

        // 1000 messages limit
        let mut t = reply_thread;
        for _ in 0..998 {
            t = t.reply(Message::new_for_testing())?;
        }
        assert!(t.reply(Message::new_for_testing()).is_err());

        Ok(())
    }
}
