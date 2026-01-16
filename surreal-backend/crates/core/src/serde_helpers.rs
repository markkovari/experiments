use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use surrealdb::sql::Thing;

// ID field serialization helpers
pub fn serialize<S>(thing: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    thing.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Try to deserialize as Thing first (what SurrealDB returns)
    let value = Option::<Thing>::deserialize(deserializer)?;
    // Extract just the ID part, not the table prefix
    Ok(value.map(|thing| thing.id.to_string()))
}

// DateTime serialization helpers
pub fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use surrealdb::sql::Datetime as SurrealDatetime;
    let surreal_dt = SurrealDatetime::from(*dt);
    surreal_dt.serialize(serializer)
}

pub fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use surrealdb::sql::Datetime as SurrealDatetime;
    let surreal_dt = SurrealDatetime::deserialize(deserializer)?;
    Ok(DateTime::from(surreal_dt))
}
