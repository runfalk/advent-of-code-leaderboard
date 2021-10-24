use anyhow::{anyhow, Result};
use std::env;

mod api;
mod config;
mod console;
mod html;
mod model;
mod parser;
mod utils;

use config::Config;

enum OutputFormat {
    Console,
    Web,
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(anyhow!(
            "Expected two arguments: web|console path/to/config.toml"
        ));
    }

    let output_format = match args[0].as_str() {
        "console" => OutputFormat::Console,
        "web" => OutputFormat::Web,
        arg => return Err(anyhow!("Unexpected second argument {:?}", arg)),
    };

    let config = Config::from_file(&args[1])?;
    let client = api::Client::new(config.session, config.cache_dir);
    for leaderboard_cfg in config.leaderboard.into_iter() {
        let leaderboard = client.fetch(leaderboard_cfg.year, leaderboard_cfg.id)?;
        let scoreboard = model::Scoreboard::from_leaderboard(&leaderboard);

        match output_format {
            OutputFormat::Console => console::render_template(&leaderboard_cfg, &scoreboard),
            OutputFormat::Web => print!("{}", html::render_template(&leaderboard_cfg, &scoreboard)),
        }
    }
    Ok(())
}
