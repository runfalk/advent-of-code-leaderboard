use anyhow::{anyhow, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde::{de::Error as SerdeError, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize)]
struct Config {
    session: String,
    leaderboards: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Day {
    #[serde(rename = "1", deserialize_with = "parse_day_progress")]
    pub part1: DateTime<Utc>,
    #[serde(rename = "2", deserialize_with = "parse_day_progress_opt", default)]
    pub part2: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    pub id: String,
    pub name: Option<String>,
    pub completion_day_level: HashMap<String, Day>,
}

#[derive(Debug, Deserialize)]
pub struct Leaderboard {
    pub event: String,
    pub members: HashMap<String, Member>,
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

impl Leaderboard {
    pub fn fetch(session: &str, year: i32, id: usize) -> Result<Self> {
        let mut cache_path = dirs::cache_dir().ok_or(anyhow!("Unable to find cache directory"))?;
        cache_path.push(&format!("aoc-leaderboard-{}-{}.json", year, id));

        let use_cached_json = if let Ok(m) = cache_path.as_path().metadata() {
            SystemTime::now()
                .duration_since(m.modified()?)
                .unwrap_or(Duration::from_secs(0))
                < Duration::from_secs(15 * 60)
        } else {
            false
        };

        let json_str = if use_cached_json {
            std::fs::read_to_string(cache_path)?
        } else {
            // TODO: Detect if session is wrong since it redirects
            let client = reqwest::blocking::Client::new();
            let rsp = client
                .get(&format!(
                    "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
                    year, id
                ))
                .header("Cookie", &format!("session={}", session))
                .send()?
                .text()?;

            let mut f = File::create(cache_path)?;
            f.write_all(rsp.as_ref())?;
            rsp
        };

        Ok(serde_json::from_str(&json_str)?)
    }
}
