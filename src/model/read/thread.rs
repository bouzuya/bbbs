use crate::model::{
    read::Message,
    shared::event::{ThreadCreated, ThreadEvent, ThreadReplied},
};

#[derive(Clone)]
pub struct Thread {
    pub id: String,
    pub messages: Vec<Message>,
    pub replies_count: u16,
    pub version: u32,
}

impl Thread {
    pub fn replay(events: Vec<ThreadEvent>) -> Self {
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
                id: thread_id.clone(),
                messages: vec![Message {
                    content,
                    created_at: at,
                    number: 1,
                    thread_id,
                }],
                replies_count: 0,
                version,
            },
            ThreadEvent::Replied(_) => {
                unreachable!("first event should be Created")
            }
        };

        for event in iter {
            thread.apply(event);
        }

        thread
    }

    pub fn apply(&mut self, event: ThreadEvent) {
        match event {
            ThreadEvent::Created(_) => {
                unreachable!("subsequent events not to be Created")
            }
            ThreadEvent::Replied(ThreadReplied {
                at,
                content,
                id: _,
                thread_id,
                version,
            }) => {
                let message = Message {
                    content,
                    created_at: at,
                    number: self.replies_count + 1,
                    thread_id,
                };
                self.messages.push(message);
                self.replies_count += 1;
                self.version = version;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::shared::event::ThreadReplied;

    use super::*;

    #[test]
    fn test_replay() {
        let events = vec![
            ThreadEvent::Created(ThreadCreated {
                at: "2023-10-01T00:00:00Z".to_string(),
                content: "Root message".to_string(),
                id: "99164b55-98d0-4e7c-98cf-95f7c43da68f".to_string(),
                thread_id: "c4ac95d6-45c7-4006-b768-2a172dee3f81".to_string(),
                version: 1,
            }),
            ThreadEvent::Replied(ThreadReplied {
                at: "2023-10-01T01:00:00Z".to_string(),
                content: "Reply message".to_string(),
                id: "4f24e399-d53a-4779-af3e-3fdfdd00f8c5".to_string(),
                thread_id: "c4ac95d6-45c7-4006-b768-2a172dee3f81".to_string(),
                version: 2,
            }),
        ];

        let thread = Thread::replay(events);
        assert_eq!(thread.id, "c4ac95d6-45c7-4006-b768-2a172dee3f81");
        assert_eq!(thread.messages.len(), 2);
        assert_eq!(thread.messages[0].content, "Root message");
        assert_eq!(thread.messages[1].content, "Reply message");
        assert_eq!(thread.replies_count, 1);
        assert_eq!(thread.version, 2);
    }
}
