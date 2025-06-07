use chrono::SubsecRound;

#[derive(Debug, thiserror::Error)]
#[error("date time error")]
pub struct DateTimeError(#[source] Box<dyn std::error::Error + Send + Sync>);

/// UNIX timestamp in milliseconds
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DateTime(i64);

impl DateTime {
    pub fn from_unix_timestamp_millis(unix_timestamp_millis: i64) -> Self {
        Self(unix_timestamp_millis)
    }

    pub fn now() -> Self {
        let chrono_date_time_utc = chrono::Utc::now();
        Self::from_unix_timestamp_millis(chrono_date_time_utc.trunc_subsecs(3).timestamp_millis())
    }

    pub fn to_unix_timestamp_millis(&self) -> i64 {
        self.0
    }
}

impl From<DateTime> for i64 {
    fn from(value: DateTime) -> Self {
        value.to_unix_timestamp_millis()
    }
}

impl From<i64> for DateTime {
    fn from(unix_timestamp_millis: i64) -> Self {
        Self::from_unix_timestamp_millis(unix_timestamp_millis)
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            chrono::DateTime::from_timestamp_millis(self.to_unix_timestamp_millis())
                .expect("unix timestamp in millis to be valid as chrono date time")
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        )
    }
}

impl std::str::FromStr for DateTime {
    type Err = DateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chrono_date_time_utc = chrono::DateTime::parse_from_rfc3339(s)
            .map(|it| it.to_utc())
            .map_err(Into::into)
            .map_err(DateTimeError)?;
        if chrono_date_time_utc != chrono_date_time_utc.trunc_subsecs(3) {
            return Err(DateTimeError(
                "DateTime must be truncated to milliseconds".into(),
            ));
        }
        Ok(Self::from_unix_timestamp_millis(
            chrono_date_time_utc.timestamp_millis(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    #[test]
    fn test_from_unix_timestamp_millis() {
        assert_eq!(
            DateTime::from_unix_timestamp_millis(0).to_string(),
            "1970-01-01T00:00:00.000Z"
        );
        assert_eq!(
            DateTime::from_unix_timestamp_millis(1).to_string(),
            "1970-01-01T00:00:00.001Z"
        );
        assert_eq!(
            DateTime::from_unix_timestamp_millis(1_000).to_string(),
            "1970-01-01T00:00:01.000Z"
        );
        assert_eq!(
            DateTime::from_unix_timestamp_millis(86_400_000).to_string(),
            "1970-01-02T00:00:00.000Z"
        );
    }

    #[test]
    fn test_now() {
        let now = DateTime::now();
        assert!(now.to_unix_timestamp_millis() >= 0);
    }

    #[test]
    fn test_to_unix_timestamp_millis() {
        let date_time = DateTime::from_unix_timestamp_millis(1_000);
        assert_eq!(date_time.to_unix_timestamp_millis(), 1_000);
    }

    #[test]
    fn test_impl_from_i64() {
        let date_time = DateTime::from(1_000_i64);
        assert_eq!(date_time.to_unix_timestamp_millis(), 1_000);
    }

    #[test]
    fn test_impl_from_date_time_for_i64() {
        let date_time = DateTime::from_unix_timestamp_millis(1_000);
        let unix_timestamp_millis = i64::from(date_time);
        assert_eq!(unix_timestamp_millis, 1_000);
    }

    #[test]
    fn test_impl_display() {
        let date_time = DateTime::from_unix_timestamp_millis(1_000);
        assert_eq!(date_time.to_string(), "1970-01-01T00:00:01.000Z");
    }

    #[test]
    fn test_impl_from_str() -> anyhow::Result<()> {
        let date_time = DateTime::from_str("1970-01-01T00:00:02.003Z")?;
        assert_eq!(date_time.to_unix_timestamp_millis(), 2_003);
        assert_eq!(
            DateTime::from_str("1970-01-01T00:00:02.003+09:00")?.to_string(),
            "1969-12-31T15:00:02.003Z"
        );

        assert!(DateTime::from_str("invalid").is_err());
        assert!(DateTime::from_str("1970-01-01T00:00:02.003004Z").is_err());
        Ok(())
    }
}
