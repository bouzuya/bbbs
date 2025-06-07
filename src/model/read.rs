#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub id: String,
}

#[derive(Clone, Eq, PartialEq)]
pub struct MessageId(pub String);
