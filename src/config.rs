use anyhow::Result;
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
