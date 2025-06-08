#[derive(Debug, thiserror::Error)]
pub enum MessageContentError {
    #[error("empty")]
    Empty,
    #[error("too long: {0}")]
    TooLong(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageContent(String);

impl MessageContent {
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let len = rng.random_range(1..=255);
        let s = rng
            .sample_iter(rand::distr::Alphanumeric)
            .map(char::from)
            .take(len)
            .collect::<String>();
        Self(s)
    }
}

impl From<MessageContent> for String {
    fn from(value: MessageContent) -> Self {
        value.0
    }
}

impl TryFrom<String> for MessageContent {
    type Error = MessageContentError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let len = value.trim().chars().count();
        if len == 0 {
            Err(MessageContentError::Empty)
        } else if len > 255 {
            Err(MessageContentError::TooLong(len))
        } else {
            assert!((1..=255).contains(&len));
            Ok(Self(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_conversion() -> anyhow::Result<()> {
        let s = "Hello, World!".to_owned();
        let content = MessageContent::try_from(s.clone())?;
        assert_eq!(String::from(content), s);

        let s = String::default();
        assert!(MessageContent::try_from(s).is_err());

        let s = " 　".to_owned();
        assert!(MessageContent::try_from(s).is_err());

        let s = "x".repeat(255);
        assert_eq!(String::from(MessageContent::try_from(s.clone())?), s);
        let s = "x".repeat(256);
        assert!(MessageContent::try_from(s).is_err());

        Ok(())
    }

    #[test]
    fn test_new_for_testing() {
        let content1 = MessageContent::new_for_testing();
        let content2 = MessageContent::new_for_testing();
        assert_ne!(content1, content2);
    }
}
