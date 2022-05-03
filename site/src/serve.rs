use crate::result::{Error, Result};
use anyhow::anyhow;
use axum::{
    http::HeaderValue,
    response::{Html, IntoResponse, Response},
};
use cap_std::{ambient_authority, fs::Dir};
use hyper::{
    header::{self, CONTENT_TYPE},
    HeaderMap, Request, StatusCode, Uri,
};
use include_dir::{include_dir, Dir as CompDir};
use lazy_static::lazy_static;
use orgish::{
    parse::{parse_n_pass, AstNode, HeaderRouting},
    treewalk::ast_to_html_string,
};
use std::str::FromStr;
use typed_html::{dom::DOMTree, html, text, unsafe_text};

use crate::ARGS;

static STATIC_DIR: CompDir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../data/static");

lazy_static! {
    static ref INDEX_URI: Uri = Uri::from_str("/index").unwrap();
    static ref ORG_DIR: Dir = Dir::open_ambient_dir(&ARGS.org_path, ambient_authority()).unwrap();
}

pub async fn fallback_handler<B>(req: Request<B>) -> Result<impl IntoResponse> {
    let uri = if req.uri() == "/" {
        &INDEX_URI
    } else {
        req.uri()
    };

    if uri.path().starts_with("/static") {
        Ok(match STATIC_DIR.get_file(&uri.path()["/static/".len()..]) {
            None => (StatusCode::NOT_FOUND, "):".to_string()).into_response(),
            Some(f) => {
                let mime = mime_guess::from_path(f.path())
                    .first()
                    .ok_or(anyhow!("couldn't guess mimetype for static file"))?
                    .to_string();
                // TODO uncomment this and make it work after axum bump to 0.5.x
                // let headers = HeaderMap::new();
                // headers.insert(
                //     CONTENT_TYPE,
                //     HeaderValue::from_str(&mime)
                // );

                (
                    StatusCode::OK,
                    f.contents_utf8()
                        .ok_or(anyhow!("file to be utf-8"))?
                        .to_owned(),
                    // headers,
                )
                    .into_response()
            }
        })
    } else {
        // we trim off the first byte since it's probably `/` and that doesn't match the hashmap keys
        // sure do hope it's not some unicode scalar that will do really weird things and make us panic
        // Ok(match ORG_DOCS.get(&uri.path()[1..]) {
        //     None => (StatusCode::NOT_FOUND, "):".to_string()).into_response(),
        //     Some(f) => Html(f.render_page_html()?).into_response(),
        // })
        let file = ORG_DIR.read_to_string("index.org")?;
        let ast = parse_n_pass(&file)?;

        for node in ast {
            match node {
                AstNode::Heading {
                    routing: Some(route),
                    title,
                    children,
                    ..
                } if route.path == &uri.path()[1..] => {
                    let html = ast_to_html_string(&children)?;
                    let text_title = format!("{:?}", title);
                    let doc: DOMTree<String> = html!(
                        <html>
                            <head>
                            <title>{ text!(text_title) }</title>
                            <link rel="stylesheet" href="/static/style.css"></link>
                            </head>

                            <body>
                                <header>
                                    <p>"hi"</p>
                                </header>

                                <main>
                                    { unsafe_text!(html) }
                                </main>
                            </body>

                        </html>
                    );

                    return Ok(Html(format!("<!DOCTYPE html>{}", doc)).into_response());
                }
                _ => {}
            }
        }

        Ok((
            StatusCode::NOT_FOUND,
            "nothing here yet.. politely go back to where you came from.".to_string(),
        )
            .into_response())
    }
}
