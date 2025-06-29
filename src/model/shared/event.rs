use std::str::FromStr as _;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThreadEvent {
    Created(ThreadCreated),
    Replied(ThreadReplied),
}

impl ThreadEvent {
    pub fn thread_id(&self) -> crate::model::shared::id::ThreadId {
        crate::model::shared::id::ThreadId::from_str(match self {
            ThreadEvent::Created(event) => &event.thread_id,
            ThreadEvent::Replied(event) => &event.thread_id,
        })
        .expect("thread_id in event to be valid")
    }

    pub fn version(&self) -> crate::model::write::Version {
        crate::model::write::Version::from(match self {
            ThreadEvent::Created(event) => event.version,
            ThreadEvent::Replied(event) => event.version,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ThreadCreated {
    pub at: String,
    pub content: String,
    pub id: String,
    pub thread_id: String,
    pub version: u32,
}

impl From<ThreadCreated> for ThreadEvent {
    fn from(event: ThreadCreated) -> Self {
        ThreadEvent::Created(event)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ThreadReplied {
    pub at: String,
    pub content: String,
    pub id: String,
    pub thread_id: String,
    pub version: u32,
}

impl From<ThreadReplied> for ThreadEvent {
    fn from(event: ThreadReplied) -> Self {
        ThreadEvent::Replied(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_event_created() -> anyhow::Result<()> {
        let at = "2023-10-01T12:00:00.000Z".to_owned();
        let content = "Hello, world!".to_owned();
        let id = "0779b098-f41d-404a-b055-36463a7c009b".to_owned();
        let thread_id = "b8392399-53a3-4f8e-8288-875448037455".to_owned();
        let version = 1;
        assert_eq!(
            serde_json::from_str::<ThreadEvent>(&format!(
                r#"
{{
    "at": "{at}",
    "content": "{content}",
    "id": "{id}",
    "kind": "created",
    "thread_id": "{thread_id}",
    "version": {version}
}}"#
            ))?,
            ThreadEvent::Created(ThreadCreated {
                at,
                content,
                id,
                thread_id,
                version,
            })
        );
        Ok(())
    }

    #[test]
    fn test_message_event_replied() -> anyhow::Result<()> {
        let at = "2023-10-01T12:00:00.000Z".to_owned();
        let content = "Reply to message".to_owned();
        let id = "0779b098-f41d-404a-b055-36463a7c009b".to_owned();
        let thread_id = "b8392399-53a3-4f8e-8288-875448037455".to_owned();
        let version = 2;
        assert_eq!(
            serde_json::from_str::<ThreadEvent>(&format!(
                r#"
{{
    "at": "{at}",
    "content": "{content}",
    "id": "{id}",
    "kind": "replied",
    "thread_id": "{thread_id}",
    "version": {version}
}}"#
            ))?,
            ThreadEvent::Replied(ThreadReplied {
                at,
                content,
                id,
                thread_id,
                version,
            })
        );
        Ok(())
    }
}
