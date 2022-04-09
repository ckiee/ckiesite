use std::net::SocketAddr;
use axum::{Router, response::{Html, IntoResponse, Response}, routing::{get, get_service}, http::StatusCode, handler::Handler};
use document::Document;
use orgish::{parse::parse_n_pass, treewalk::ast_to_html_string};
use clap::Parser;
use template::make_article_html;
use thiserror::Error;
use tracing::{debug, error};
use tower_http::services::ServeDir;

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

    let app = Router::new()
        .route("/", get(root_unauthenticated))
        // TODO why doesn't this use the 404 fallback?
        .nest("/static", get_service(ServeDir::new("./data/static")).handle_error(|e: std::io::Error| async move {
            error!("io error while serving static data: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "io error while serving static data"
            )
        }))
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

async fn root_unauthenticated() -> Result<impl IntoResponse> {
    let doc = Document::from_org_file("data/index.org").await?;
    Ok(Html(doc.render_page_html()?))
}

async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "hi.. nothing here.")
}
