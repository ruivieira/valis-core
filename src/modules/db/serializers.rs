use std::str::FromStr;

use chrono::{DateTime, Duration, NaiveDate, ParseError, Utc};
use rusqlite::types::FromSqlError;
use rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct SerializableDateTime(DateTime<Utc>);

impl Serialize for SerializableDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.0.to_rfc3339();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for SerializableDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = DateTime::parse_from_rfc3339(&s)
            .map_err(Error::custom)?
            .with_timezone(&Utc);
        Ok(SerializableDateTime(dt))
    }
}

impl FromSql for SerializableDateTime {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(s) => {
                let s = String::from_utf8(s.to_vec())
                    .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?;
                DateTime::from_str(&s)
                    .map_err(|_| FromSqlError::InvalidType)
                    .map(|dt| SerializableDateTime(dt))
            }
            ValueRef::Blob(b) => {
                let s = String::from_utf8(b.to_vec())
                    .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?;
                DateTime::parse_from_rfc3339(&s)
                    .map_err(|_| rusqlite::types::FromSqlError::InvalidType)
                    .map(|dt| SerializableDateTime(dt.with_timezone(&Utc)))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl SerializableDateTime {
    pub fn parse_from_rfc3339(s: &str) -> Result<SerializableDateTime, ParseError> {
        let dt = DateTime::parse_from_rfc3339(s)?;
        Ok(SerializableDateTime(dt.with_timezone(&Utc)))
    }
    pub fn with_timezone(&self, tz: &Utc) -> SerializableDateTime {
        SerializableDateTime(self.0.with_timezone(tz))
    }
    pub fn now() -> Self {
        SerializableDateTime(Utc::now())
    }
    pub fn to_string(&self) -> String {
        self.0.to_rfc3339()
    }
    pub fn add_weeks(&self, weeks: i64) -> Self {
        SerializableDateTime(self.0 + Duration::weeks(weeks))
    }
    // Create a new SerializableDateTime from a string
    pub fn from_str(date: &str) -> Result<Self, chrono::format::ParseError> {
        let naive_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
        let datetime = DateTime::from_utc(naive_date.and_hms(0, 0, 0), Utc);
        Ok(SerializableDateTime(datetime))
    }
}
