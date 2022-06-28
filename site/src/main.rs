use axum::{error_handling::HandleError, Router};
use clap::Parser;
use hyper::StatusCode;
use lazy_static::lazy_static;
use std::{net::SocketAddr, path::PathBuf};
use tower::service_fn;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::serve::fill_caches;

pub mod serve;

/// Frontend for orgish to serve website
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// HTTP listening port
    #[clap(short, long, default_value_t = 13337)]
    port: u16,

    /// Listen on all interfaces instead of loopback
    #[clap(short, long)]
    everywhere: bool,

    /// Path to the content folder
    content_path: PathBuf,

    /// Path to the static folder
    static_path: PathBuf,

    /// Whether to cache index.org. This saves around ~20ms per request.
    #[clap(short, long)]
    cache_org: bool,
}

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

#[tokio::main(worker_threads = 1)]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default({
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))?;
        tracing_subscriber::registry()
            .with(tracing_tracy::TracyLayer::new())
            .with(filter_layer)
            .with(tracing_subscriber::fmt::layer()) // set RUST_LOG=debug
    })?;

    if !ARGS.cache_org {
        warn!("--cache-org is not set! expect degraded performance");
    }

    // Warm the server up.
    serve::fill_caches();

    let app = Router::new().fallback(HandleError::new(
        service_fn(serve::fallback_handler),
        handle_anyhow_error,
    ));

    let addr = SocketAddr::from((
        if ARGS.everywhere {
            [0, 0, 0, 0]
        } else {
            [127, 0, 0, 1]
        },
        ARGS.port,
    ));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

pub async fn handle_anyhow_error(err: anyhow::Error) -> (StatusCode, String) {
    error!("error while serving request: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "500 oopsie doopsie".to_string(),
    )
}
