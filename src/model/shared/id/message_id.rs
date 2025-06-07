#[derive(Debug, thiserror::Error)]
#[error("message id error")]
pub struct MessageIdError(#[source] Box<dyn std::error::Error + Send + Sync>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageId(uuid::Uuid);

impl MessageId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::str::FromStr for MessageId {
    type Err = MessageIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(s)
            .map_err(Into::into)
            .map_err(MessageIdError)?;
        if uuid.get_version_num() != 4 {
            return Err(MessageIdError("invalid UUID version".into()));
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_generate() {
        let id1 = MessageId::generate();
        let id2 = MessageId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_impl_display() {
        let id = MessageId::generate();
        let s = id.to_string();
        assert_eq!(s.len(), 36);
    }

    #[test]
    fn test_impl_from_str() -> anyhow::Result<()> {
        let id = MessageId::generate();
        assert_eq!(MessageId::from_str(&id.to_string())?, id);
        assert_eq!(
            MessageId::from_str("123").unwrap_err().to_string(),
            "message id error"
        );
        Ok(())
    }
}
