use std::{net::SocketAddr, str::FromStr};
use axum::{Router, response::{Html, IntoResponse, Response}, routing::{get, get_service}, http::{StatusCode, uri::PathAndQuery}, handler::Handler, extract::Path, middleware::{Next, self}};
use document::Document;
use hyper::{Request, Uri};
use orgish::{parse::parse_n_pass, treewalk::ast_to_html_string};
use clap::Parser;
use template::make_article_html;
use thiserror::Error;
use tower::Layer;
use tracing::{debug, error};
use tower_http::services::ServeDir;
use anyhow::anyhow;

mod template;
pub mod document;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error)
}

type Result<A> = std::result::Result<A, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("error while serving request: {:#?}", &self);
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error ):").into_response()
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
    everywhere: bool
}

// TODO compile all the posts' raw content and cache that in memory
// TODO serve posts
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt::init(); // loggy log, set RUST_LOG=debug

    let error_handler = |e: std::io::Error| async move {
            error!("io error while serving static data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "io error while serving static data"
            )
        };

    let app = Router::new()
        .nest("/static", get_service(ServeDir::new("./data/static")).handle_error(error_handler))
        .fallback(fallback_handler.into_service());

    let addr = SocketAddr::from((if args.everywhere {
        [0, 0, 0, 0]
    } else {
        [127, 0, 0, 1]
    }, args.port));
    debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn index_rewrite<B>(mut req: Request<B>, next: Next<B>) -> Result<impl IntoResponse> {
    debug!("hi hi index rew {}", req.uri());
    if req.uri().path() == "/" {
        let mut parts = req.uri().clone().into_parts();
        parts.path_and_query = Some(PathAndQuery::from_static("/index"));
        *req.uri_mut() = Uri::from_parts(parts).map_err(|e| anyhow!("Uri::from_parts"))?;
        debug!("in {}", req.uri());
    }

    Ok(next.run(req).await)
}

async fn org_file(
    Path(key): Path<String>,
    // Extension(state): Extension<SharedState>,
) -> Result<impl IntoResponse> {
    dbg!(&key);
    Ok("hah a you want a page thats very funny")
}


async fn fallback_handler<B>(req: Request<B>) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "hi.. 404! nothing here.")
}
