use anyhow::Result;
use axum::response::Response;
use clap::Parser;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

#[derive(Debug, Parser)]
enum Opt {
    /// Start a webserver that serves the leaderboard
    Server {
        /// TOML configuration file
        config: PathBuf,

        /// Bind address and port
        #[clap(default_value = "0.0.0.0:3000")]
        host: String,
    },

    /// Print the current standings of all leaderboards and exit
    Console {
        /// TOML configuration file
        config: PathBuf,
    },
}

impl Opt {
    fn config_path(&self) -> &Path {
        match self {
            Opt::Server { ref config, .. } => config,
            Opt::Console { ref config, .. } => config,
        }
    }
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
type AocClient = Arc<Mutex<api::Client>>;

async fn get_leaderboard(
    extract::Path(slug): extract::Path<String>,
    extract::Extension(cfg): extract::Extension<Arc<HashMap<String, LeaderboardConfig>>>,
    extract::Extension(metadata): extract::Extension<
        Arc<HashMap<i32, HashMap<usize, MemberMetadata>>>,
    >,
    extract::Extension(client): extract::Extension<AocClient>,
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
            .fetch(leaderboard_cfg.year, leaderboard_cfg.id)
            .await?
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
            Self::InternalError(_) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error",
            ),
        };
        (status, error_message).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::parse();
    let config = Config::from_file(opts.config_path())?;

    match opts {
        Opt::Server { host, .. } => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(
                    std::env::var("RUST_LOG").unwrap_or_else(|_| {
                        "advent_of_code_leaderboard=debug,tower_http=debug".into()
                    }),
                ))
                .with(tracing_subscriber::fmt::layer())
                .init();
            let client = api::Client::new(config.session, config.cache_dir);
            let metadata = config.metadata;
            let config = config
                .leaderboard
                .into_iter()
                .map(|l| (l.slug.clone(), l))
                .collect::<HashMap<_, _>>();

            let app = Router::new()
                .route("/:slug", routing::get(get_leaderboard))
                .layer(TraceLayer::new_for_http())
                .layer(Extension(Arc::new(config)))
                .layer(Extension(Arc::new(metadata)))
                .layer(Extension(Arc::new(Mutex::new(client))));

            let bind: SocketAddr = host.parse()?;
            tracing::info!("Listening on {}", &bind);
            let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Opt::Console { .. } => {
            let client = api::Client::new(config.session, config.cache_dir);
            let empty_metadata = HashMap::new();
            for leaderboard_cfg in config.leaderboard.into_iter() {
                let leaderboard = client
                    .fetch(leaderboard_cfg.year, leaderboard_cfg.id)
                    .await?;
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
