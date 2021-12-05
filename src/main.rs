use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

use axum::{
    body, extract, http, response, response::IntoResponse, routing, AddExtensionLayer, Router,
};

mod api;
mod config;
mod console;
mod html;
mod model;
mod parser;
mod utils;

use config::{Config, LeaderboardConfig};

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

impl<T> From<T> for WebError where T: Into<anyhow::Error> {
    fn from(error: T) -> Self {
        Self::InternalError(error.into())
    }
}

// API client that is shared across all requests (makes sure that we don't refresh simultaneously)
type OurClient = Arc<Mutex<api::Client>>;

async fn get_leaderboard(
    extract::Path(slug): extract::Path<String>,
    extract::Extension(cfg): extract::Extension<Arc<HashMap<String, LeaderboardConfig>>>,
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
            .unwrap()
            .fetch(leaderboard_cfg.year, leaderboard_cfg.id)?
    };
    let scoreboard = model::Scoreboard::from_leaderboard(&leaderboard);

    Ok(response::Html(html::render_template(
        leaderboard_cfg,
        &scoreboard,
    )))
}

impl IntoResponse for WebError {
    type Body = body::Full<body::Bytes>;
    type BodyError = std::convert::Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let (status, error_message) = match self {
            Self::NotFound => (http::StatusCode::NOT_FOUND, "404 Not Found"),
            Self::InternalError(e) => {
                println!("{}", e);
                (http::StatusCode::INTERNAL_SERVER_ERROR, "500 Internal Server Error")
            },
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
            let config = config
                .leaderboard
                .into_iter()
                .map(|l| (l.slug.clone(), l))
                .collect::<HashMap<_, _>>();

            // build our application with a single route
            let app = Router::new()
                .route("/:slug", routing::get(get_leaderboard))
                .layer(AddExtensionLayer::new(Arc::new(config)))
                .layer(AddExtensionLayer::new(Arc::new(Mutex::new(client))));

            // run it with hyper on localhost:3000
            axum::Server::bind(&host.parse()?)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        Opt::Console { .. } => {
            let client = api::Client::new(config.session, config.cache_dir);
            for leaderboard_cfg in config.leaderboard.into_iter() {
                let leaderboard = client.fetch(leaderboard_cfg.year, leaderboard_cfg.id)?;
                let scoreboard = model::Scoreboard::from_leaderboard(&leaderboard);
                console::render_template(&leaderboard_cfg, &scoreboard);
            }
        }
    };

    Ok(())
}
