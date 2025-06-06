#[derive(Clone)]
pub struct Message {
    pub content: String,
    pub id: MessageId,
}

impl Message {
    pub fn create(content: String) -> Self {
        Self {
            content,
            // FIXME: generate a unique ID
            id: MessageId("123".to_owned()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageId(pub String);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u32);
