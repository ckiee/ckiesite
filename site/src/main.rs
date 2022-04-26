use anyhow::anyhow;
use axum::{
    handler::Handler,
    response::{Html, IntoResponse, Response},
    Router,
};
use clap::Parser;
use hyper::{Request, StatusCode, Uri};
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use site_common::document::Document;
use std::{collections::HashMap, net::SocketAddr, str::FromStr};
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

type Result<A> = std::result::Result<A, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("error while serving request: {:#?}", &self);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal server error ):",
        )
            .into_response()
    }
}

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

    let app = Router::new().fallback(fallback_handler.into_service());

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

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../data/static");

lazy_static! {
    static ref INDEX_ORG_URI: Uri = Uri::from_str("/index.org").unwrap();
    static ref ORG_DOCS: HashMap<String, Document> = org_doc::org_docs!();
}

async fn fallback_handler<B>(req: Request<B>) -> Result<impl IntoResponse> {
    let uri = if req.uri() == "/" {
        &INDEX_ORG_URI
    } else {
        req.uri()
    };

    if uri.path().starts_with("/static") {
        Ok(match STATIC_DIR.get_file(&uri.path()["/static/".len()..]) {
            None => (StatusCode::NOT_FOUND, "):".to_string()).into_response(),
            Some(f) => (
                StatusCode::OK,
                f.contents_utf8()
                    .ok_or(anyhow!("file to be utf-8"))?
                    .to_owned(),
            )
                .into_response(),
        })
    } else {
        // we trim off the first byte since it's probably `/` and that doesn't match the hashmap keys
        // sure do hope it's not some unicode scalar that will do really weird things and make us panic
        Ok(match ORG_DOCS.get(&uri.path()[1..]) {
            None => (StatusCode::NOT_FOUND, "):".to_string()).into_response(),
            Some(f) => Html(f.render_page_html()?).into_response(),
        })
    }
}
