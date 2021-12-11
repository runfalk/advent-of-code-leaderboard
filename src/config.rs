use anyhow::Result;
use serde::de::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub session: String,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    pub leaderboard: Vec<LeaderboardConfig>,

    // pub metadata: Option<Vec<Metadata>>,
    #[serde(deserialize_with = "parse_metadata")]
    pub metadata: HashMap<i32, HashMap<usize, MemberMetadata>>,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardConfig {
    pub id: usize,
    pub name: String,
    pub slug: String,
    pub code: String,
    pub year: i32, // We use i32 since that's what chrono expects

    #[serde(default)]
    pub repositories: HashMap<usize, String>,

    #[serde(default)]
    pub header: String,
}

#[derive(Debug)]
pub struct Metadata {
    pub year: i32,
    pub members: HashMap<usize, MemberMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct MemberMetadata {
    pub repository: Option<String>,
}

fn parse_metadata<'de, D>(de: D) -> Result<HashMap<i32, HashMap<usize, MemberMetadata>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut metadata = HashMap::new();

    let metadata_tables: Vec<toml::value::Table> = Deserialize::deserialize(de)?;
    for mut raw_metadata in metadata_tables {
        let year: i32 = match raw_metadata
            .remove("year")
            .ok_or_else(|| D::Error::custom("Missing year field in metadata"))?
        {
            toml::Value::Integer(i) => {
                i.try_into().map_err(|_| D::Error::custom("Invalid year"))?
            }
            _ => return Err(D::Error::custom("asdf")),
        };

        let mut members: HashMap<usize, MemberMetadata> = HashMap::new();
        for (member_id_str, m) in raw_metadata {
            // We can't detect duplicate keys because toml overrides duplicates before we get here
            members.insert(
                member_id_str
                    .parse()
                    .map_err(|_| D::Error::custom("Member must be an integer"))?,
                Deserialize::deserialize(m).map_err(|e| D::Error::custom(e.to_string()))?,
            );
        }

        if metadata.insert(year, members).is_some() {
            return Err(D::Error::custom(format!(
                "Year must be unique for all metadata tables (got {} twice)",
                year
            )));
        }
    }
    Ok(metadata)
}

/// Return a default cache directory, and if unable to determine one, try the
/// current working directory, and if that for some god forsaken reason fails
/// we use a temporary directory.
fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(env::temp_dir)
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(p: P) -> Result<Config> {
        let mut config_str = String::new();
        std::fs::File::open(p)?.read_to_string(&mut config_str)?;
        Ok(toml::from_str(&config_str)?)
    }
}
