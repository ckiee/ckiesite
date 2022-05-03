use axum::{
    handler::Handler,
    Router,
};
use clap::Parser;
use hyper::{StatusCode};
use std::{net::SocketAddr, path::{Path, PathBuf}};
use tracing::{debug, error};
use lazy_static::lazy_static;


pub mod document;
pub mod serve;
pub mod result;

use result::Result;

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

    /// Path to the org folder
    org_path: PathBuf
}

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

// TODO compile all the posts' raw content and cache that in memory
// TODO serve posts
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init(); // loggy log, set RUST_LOG=debug

    let _error_handler = |e: std::io::Error| async move {
        error!("io error while serving static data: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "io error while serving static data",
        )
    };

    let app = Router::new().fallback(serve::fallback_handler.into_service());

    let addr = SocketAddr::from((
        if ARGS.everywhere {
            [0, 0, 0, 0]
        } else {
            [127, 0, 0, 1]
        },
        ARGS.port,
    ));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
