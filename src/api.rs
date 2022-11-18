use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::parser::Leaderboard;

pub struct Client {
    session: String,
    cache_dir: PathBuf,
}

impl Client {
    pub fn new<S: Into<String>, P: Into<PathBuf>>(session: S, cache_dir: P) -> Self {
        Self {
            session: session.into(),
            cache_dir: cache_dir.into(),
        }
    }

    pub async fn fetch(&self, year: i32, id: usize) -> Result<Leaderboard> {
        let cache_path = self
            .cache_dir
            .join(&format!("aoc-leaderboard-{}-{}.json", year, id));

        // We're only allowed to fetch the JSON once every 15 min. Check if we have a cached
        // version before trying
        let use_cached_json = if let Ok(m) = cache_path.as_path().metadata() {
            let last_modified = SystemTime::now()
                .duration_since(m.modified()?)
                .unwrap_or(Duration::ZERO);
            last_modified < Duration::from_secs(15 * 60)
        } else {
            false
        };

        let json_str = if use_cached_json {
            std::fs::read_to_string(cache_path)?
        } else {
            // TODO: Detect if session is wrong since it redirects
            let client = reqwest::Client::new();
            let rsp = client
                .get(&format!(
                    "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
                    year, id
                ))
                .header("Cookie", &format!("session={}", &self.session))
                .send()
                .await?
                .text()
                .await?;

            // Save updated content in the cache
            let mut f = File::create(cache_path)?;
            f.write_all(rsp.as_ref())?;

            rsp
        };

        Ok(serde_json::from_str(&json_str)?)
    }
}
