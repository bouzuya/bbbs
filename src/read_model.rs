#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub id: MessageId,
}

#[derive(Clone, Eq, PartialEq)]
pub struct MessageId(pub String);
