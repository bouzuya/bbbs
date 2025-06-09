#[derive(Debug, thiserror::Error)]
#[error("thread id error")]
pub struct ThreadIdError(#[source] Box<dyn std::error::Error + Send + Sync>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadId(uuid::Uuid);

impl ThreadId {
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::str::FromStr for ThreadId {
    type Err = ThreadIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::parse_str(s)
            .map_err(Into::into)
            .map_err(ThreadIdError)?;
        if uuid.get_version_num() != 4 {
            return Err(ThreadIdError("invalid UUID version".into()));
        }
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for ThreadId {
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
        let id1 = ThreadId::generate();
        let id2 = ThreadId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_impl_display() {
        let id = ThreadId::generate();
        let s = id.to_string();
        assert_eq!(s.len(), 36);
    }

    #[test]
    fn test_impl_from_str() -> anyhow::Result<()> {
        let id = ThreadId::generate();
        assert_eq!(ThreadId::from_str(&id.to_string())?, id);
        assert_eq!(
            ThreadId::from_str("123").unwrap_err().to_string(),
            "thread id error"
        );
        Ok(())
    }
}
