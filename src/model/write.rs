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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageId(uuid::Uuid);

impl MessageId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_generate() {
        let id1 = MessageId::generate();
        let id2 = MessageId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_message_create() {
        let content = "Hello, world!".to_string();
        let message = Message::create(content.clone());
        assert_eq!(message.content, content);
        assert!(!message.id.to_string().is_empty());
    }
}
