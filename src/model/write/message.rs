use crate::model::shared::id::MessageId;
use crate::model::write::MessageContent;
use crate::utils::date_time::DateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub content: MessageContent,
    pub created_at: DateTime,
    pub id: MessageId,
}

impl Message {
    pub fn create(content: MessageContent) -> Self {
        Self {
            content,
            created_at: DateTime::now(),
            id: MessageId::generate(),
        }
    }

    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        Self {
            content: MessageContent::new_for_testing(),
            created_at: DateTime::now(),
            id: MessageId::generate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let content = MessageContent::new_for_testing();
        let message = Message::create(content.clone());
        assert_eq!(message.content, content);
        assert!(!message.id.to_string().is_empty());
    }

    #[test]
    fn test_new_for_testing() {
        let message1 = Message::new_for_testing();
        let message2 = Message::new_for_testing();
        assert_ne!(message1, message2);
    }
}
