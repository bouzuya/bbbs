#[derive(Debug, thiserror::Error)]
#[error("event id error")]
pub struct EventIdError(#[source] Box<dyn std::error::Error + Send + Sync>);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EventId(uuid::Uuid);

impl EventId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::str::FromStr for EventId {
    type Err = EventIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(s)
            .map_err(Into::into)
            .map_err(EventIdError)?;
        if uuid.get_version_num() != 4 {
            return Err(EventIdError("invalid UUID version".into()));
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for EventId {
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
        let id1 = EventId::generate();
        let id2 = EventId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_impl_display() {
        let id = EventId::generate();
        let s = id.to_string();
        assert_eq!(s.len(), 36);
    }

    #[test]
    fn test_impl_from_str() -> anyhow::Result<()> {
        let id = EventId::generate();
        assert_eq!(EventId::from_str(&id.to_string())?, id);
        assert_eq!(
            EventId::from_str("123").unwrap_err().to_string(),
            "event id error"
        );
        Ok(())
    }
}
