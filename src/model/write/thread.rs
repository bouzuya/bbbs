use std::str::FromStr;

use crate::model::shared::event::ThreadCreated;
use crate::model::shared::event::ThreadEvent;
use crate::model::shared::event::ThreadReplied;
use crate::model::shared::id::EventId;
use crate::model::shared::id::ThreadId;
use crate::model::write::Message;
use crate::model::write::MessageContent;
use crate::model::write::Version;
use crate::utils::date_time::DateTime;

#[derive(Debug, thiserror::Error)]
#[error("thread error")]
pub struct ThreadError(#[source] Box<dyn std::error::Error + Send + Sync>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Thread {
    id: ThreadId,
    message_count: usize,
    root_message: Message,
    version: Version,
}

impl Thread {
    pub fn create(message: Message) -> Result<(Self, Vec<ThreadEvent>), ThreadError> {
        let id = ThreadId::generate();
        let version = Version::initial();

        let event = ThreadEvent::from(ThreadCreated {
            at: DateTime::now().to_string(),
            content: String::from(message.content.clone()),
            id: EventId::generate().to_string(),
            thread_id: id.to_string(),
            version: u32::from(version),
        });
        Ok((
            Self {
                id,
                message_count: 1,
                root_message: message,
                version,
            },
            vec![event],
        ))
    }

    pub fn replay(events: &[ThreadEvent]) -> Self {
        let mut iter = events.into_iter();

        let first_event = iter.next().expect("events not to be empty");
        let mut thread = match first_event {
            ThreadEvent::Created(ThreadCreated {
                at,
                content,
                id: _,
                thread_id,
                version,
            }) => Self {
                id: ThreadId::from_str(&thread_id).expect("thread id in event to be valid"),
                message_count: 1,
                root_message: Message {
                    content: MessageContent::try_from(content.to_owned())
                        .expect("message content in event to be valid"),
                    created_at: DateTime::from_str(&at)
                        .expect("message created_at in event to be valid"),
                },
                version: Version::from(*version),
            },
            ThreadEvent::Replied(_) => {
                unreachable!("first event should be Created")
            }
        };

        for event in iter {
            match event {
                ThreadEvent::Created(_) => {
                    unreachable!("subsequent events not to be Created")
                }
                ThreadEvent::Replied(ThreadReplied {
                    at: _,
                    content: _,
                    id: _,
                    thread_id: _,
                    version,
                }) => {
                    thread.message_count += 1;
                    thread.version = Version::from(*version);
                }
            }
        }

        thread
    }

    pub fn id(&self) -> &ThreadId {
        &self.id
    }

    pub fn reply(&self, message: Message) -> Result<(Self, Vec<ThreadEvent>), ThreadError> {
        if self.message_count == 1000 {
            return Err(ThreadError(
                "Thread has reached the maximum number of messages".into(),
            ));
        }
        let version = self.version.next();
        let event = ThreadEvent::from(ThreadReplied {
            at: DateTime::now().to_string(),
            content: String::from(message.content.clone()),
            id: EventId::generate().to_string(),
            thread_id: self.id.to_string(),
            version: u32::from(version),
        });
        Ok((
            Self {
                id: self.id.clone(),
                message_count: self.message_count + 1,
                root_message: self.root_message.clone(),
                version,
            },
            vec![event],
        ))
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
        let (created, _events) = Thread::create(message.clone())?;
        assert!(!created.id().to_string().is_empty());
        assert_eq!(created.version(), Version::initial());
        Ok(())
    }

    #[test]
    fn test_replay() -> anyhow::Result<()> {
        let message = Message::new_for_testing();
        let (created, created_events) = Thread::create(message.clone())?;
        let (replied, replied_events) = created.reply(Message::new_for_testing())?;
        let replayed = Thread::replay(
            &created_events
                .into_iter()
                .chain(replied_events.into_iter())
                .collect::<Vec<_>>(),
        );

        assert_eq!(replayed.id(), replied.id());
        assert_eq!(replayed.version(), replied.version());
        assert_eq!(replayed.message_count, replied.message_count);

        Ok(())
    }

    #[test]
    fn test_reply() -> anyhow::Result<()> {
        let root_message = Message::new_for_testing();
        let (created, _events) = Thread::create(root_message.clone())?;
        let reply_message = Message::new_for_testing();
        let (replied, _events) = created.reply(reply_message.clone())?;

        assert_eq!(replied.id(), created.id());
        assert_eq!(replied.version(), created.version().next());

        // 1000 messages limit
        let mut t = replied;
        for _ in 0..998 {
            (t, _) = t.reply(Message::new_for_testing())?;
        }
        assert!(t.reply(Message::new_for_testing()).is_err());

        Ok(())
    }
}
