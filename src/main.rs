use anyhow::Result;
use axum::response::Response;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

use axum::{extract, http, response, response::IntoResponse, routing, Extension, Router};

mod api;
mod config;
mod console;
mod html;
mod model;
mod parser;
mod utils;

use config::{Config, LeaderboardConfig};

use self::config::MemberMetadata;

#[derive(Debug, StructOpt)]
enum Opt {
    /// Start a webserver that serves the leaderboard
    Server {
        /// TOML configuration file
        config: PathBuf,

        #[structopt(default_value = "0.0.0.0:3000")]
        host: String,
    },

    /// Print the current standings of all leaderboards and exit
    Console {
        /// TOML configuration file
        config: PathBuf,
    },
}

#[derive(Debug)]
enum WebError {
    NotFound,
    InternalError(anyhow::Error),
}

impl<T> From<T> for WebError
where
    T: Into<anyhow::Error>,
{
    fn from(error: T) -> Self {
        Self::InternalError(error.into())
    }
}

// API client that is shared across all requests (makes sure that we don't refresh simultaneously)
type OurClient = Arc<Mutex<api::Client>>;

async fn get_leaderboard(
    extract::Path(slug): extract::Path<String>,
    extract::Extension(cfg): extract::Extension<Arc<HashMap<String, LeaderboardConfig>>>,
    extract::Extension(metadata): extract::Extension<
        Arc<HashMap<i32, HashMap<usize, MemberMetadata>>>,
    >,
    extract::Extension(client): extract::Extension<OurClient>,
) -> Result<response::Html<String>, WebError> {
    let leaderboard_cfg = if let Some(cfg) = cfg.get(&slug) {
        cfg
    } else {
        return Err(WebError::NotFound);
    };

    let leaderboard = {
        client
            .lock()
            .await
            .fetch(leaderboard_cfg.year, leaderboard_cfg.id).
            await?
    };
    let scoreboard = model::Scoreboard::from_leaderboard(&leaderboard);

    let empty_metadata = HashMap::new();
    let metadata = metadata
        .get(&leaderboard_cfg.year)
        .unwrap_or(&empty_metadata);

    Ok(response::Html(html::render_template(
        leaderboard_cfg,
        metadata,
        &scoreboard,
    )))
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::NotFound => (http::StatusCode::NOT_FOUND, "404 Not Found"),
            Self::InternalError(e) => {
                println!("{}", e);
                (
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    "500 Internal Server Error",
                )
            }
        };
        (status, error_message).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();
    let config = Config::from_file(match opts {
        Opt::Server { ref config, .. } => config,
        Opt::Console { ref config, .. } => config,
    })?;

    match opts {
        Opt::Server { host, .. } => {
            let client = api::Client::new(config.session, config.cache_dir);
            let metadata = config.metadata;
            let config = config
                .leaderboard
                .into_iter()
                .map(|l| (l.slug.clone(), l))
                .collect::<HashMap<_, _>>();

            // build our application with a single route
            let app = Router::new()
                .route("/:slug", routing::get(get_leaderboard))
                .layer(Extension(Arc::new(config)))
                .layer(Extension(Arc::new(metadata)))
                .layer(Extension(Arc::new(Mutex::new(client))));

            // run it with hyper on localhost:3000
            axum::Server::bind(&host.parse()?)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        Opt::Console { .. } => {
            let client = api::Client::new(config.session, config.cache_dir);
            let empty_metadata = HashMap::new();
            for leaderboard_cfg in config.leaderboard.into_iter() {
                let leaderboard = client.fetch(leaderboard_cfg.year, leaderboard_cfg.id).await?;
                let scoreboard = model::Scoreboard::from_leaderboard(&leaderboard);
                let metadata = config
                    .metadata
                    .get(&leaderboard_cfg.year)
                    .unwrap_or(&empty_metadata);
                console::render_template(&leaderboard_cfg, metadata, &scoreboard);
            }
        }
    };

    Ok(())
}
