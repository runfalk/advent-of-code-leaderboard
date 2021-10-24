use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use serde::{de::Error as SerdeError, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// Parser for the Advent of Code leaderboard JSON. We ignore keys that are not interesting to this
/// leaderboard calculation.
///
/// Example:
///
/// {
///     "event": "2020",
///     "owner_id": "273465",
///     "members": {
///         "273465": {
///             "id": "273465",
///             "last_star_ts": 1608891747,
///             "completion_day_level": {
///                 "4": {
///                     "1": {"get_star_ts": 1607071035},
///                     "2": {"get_star_ts": 1607072594}
///                 }
///             },
///             "name": "Andreas Runfalk",
///             "stars": 50,
///             "global_score": 0,
///             "local_score": 751
///         }
///     }
/// }
///
/// There are some gotchas.
///
/// - If a user has not completed any stars for a day, it'll not be in the dictionary
/// - If a user has completed the first but not the second task, the second task will be absent
///   from the from the dictionary.
/// - If a user is marked as anonymous they'll not have a name key.
/// - There is no way to get the associated GitHub of a user through the JSON.

#[derive(Debug, Deserialize)]
pub struct Day {
    #[serde(rename = "1", deserialize_with = "parse_day_progress")]
    pub part1: DateTime<Utc>,
    #[serde(rename = "2", deserialize_with = "parse_day_progress_opt", default)]
    pub part2: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    #[serde(deserialize_with = "parse_json_number")]
    pub id: usize,
    pub name: Option<String>,
    pub completion_day_level: HashMap<usize, Day>,
}

#[derive(Debug, Deserialize)]
pub struct Leaderboard {
    #[serde(deserialize_with = "parse_json_number")]
    pub event: i32,
    pub members: HashMap<usize, Member>,
}

fn parse_json_number<'de, D, T, E>(de: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr<Err = E>,
    E: std::fmt::Display,
{
    let value: Value = Deserialize::deserialize(de)?;
    match value {
        Value::Number(n) => Ok(n.to_string().parse().map_err(D::Error::custom)?),
        Value::String(n) => Ok(n.parse().map_err(D::Error::custom)?),
        _ => Err(D::Error::custom("expected number")),
    }
}

fn parse_json_value_ts<E: serde::de::Error>(value: &Value) -> Result<DateTime<Utc>, E> {
    match value {
        Value::Number(n) => Ok(Utc.timestamp(n.as_i64().ok_or_else(|| E::custom("overflow"))?, 0)),
        Value::String(n) => Ok(Utc.timestamp(n.parse().map_err(E::custom)?, 0)),
        _ => Err(E::custom("invalid timstamp")),
    }
}

fn parse_day_progress<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
    D::Error: serde::de::Error,
{
    if let Value::Object(m) = Value::deserialize(deserializer)? {
        let ts_value = m
            .get("get_star_ts")
            .ok_or_else(|| D::Error::custom("invalid day progress"))?;
        parse_json_value_ts(&ts_value)
    } else {
        Err(D::Error::custom("invalid day progress"))
    }
}

fn parse_day_progress_opt<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
    D::Error: serde::de::Error,
{
    Some(parse_day_progress(deserializer)).transpose()
}
