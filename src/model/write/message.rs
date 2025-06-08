use crate::model::shared::id::MessageId;
use crate::model::write::MessageContent;

#[derive(Clone)]
pub struct Message {
    pub content: MessageContent,
    pub id: MessageId,
}

impl Message {
    pub fn create(content: MessageContent) -> Self {
        Self {
            content,
            id: MessageId::generate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_create() {
        let content = MessageContent::new_for_testing();
        let message = Message::create(content.clone());
        assert_eq!(message.content, content);
        assert!(!message.id.to_string().is_empty());
    }
}
