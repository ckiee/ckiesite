
use axum::{
    handler::Handler,
    Router,
};
use clap::Parser;
use hyper::{StatusCode};
use std::{net::SocketAddr};
use tracing::{debug, error};


pub mod document;
pub mod serve;
pub mod result;

use result::Result;

/// Frontend for orgish to serve website
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// HTTP listening port
    #[clap(short, long, default_value_t = 13337)]
    port: u16,

    /// Listen on all interfaces instead of loopback
    #[clap(short, long)]
    everywhere: bool,
}

// TODO compile all the posts' raw content and cache that in memory
// TODO serve posts
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
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
        if args.everywhere {
            [0, 0, 0, 0]
        } else {
            [127, 0, 0, 1]
        },
        args.port,
    ));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
