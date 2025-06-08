use crate::model::shared::id::MessageId;

#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub id: MessageId,
}

impl Message {
    pub fn create(content: String) -> Self {
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
        let content = "Hello, world!".to_string();
        let message = Message::create(content.clone());
        assert_eq!(message.content, content);
        assert!(!message.id.to_string().is_empty());
    }
}
