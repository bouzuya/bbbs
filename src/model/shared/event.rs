#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MessageEvent {
    Created(MessageCreated),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MessageCreated {
    pub at: String,
    pub content: String,
    pub id: String,
    pub message_id: String,
    pub version: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_event_created() -> anyhow::Result<()> {
        assert_eq!(
            serde_json::from_str::<MessageEvent>(
                r#"
{
    "at": "2023-10-01T12:00:00.000Z",
    "content": "Hello, world!",
    "id": "0779b098-f41d-404a-b055-36463a7c009b",
    "kind": "created",
    "message_id": "524313c5-6810-4f5d-a2a1-66a2224c74dc",
    "version": 1
}"#,
            )?,
            MessageEvent::Created(MessageCreated {
                at: "2023-10-01T12:00:00.000Z".to_owned(),
                content: "Hello, world!".to_owned(),
                id: "0779b098-f41d-404a-b055-36463a7c009b".to_owned(),
                message_id: "524313c5-6810-4f5d-a2a1-66a2224c74dc".to_owned(),
                version: 1,
            })
        );
        Ok(())
    }
}
