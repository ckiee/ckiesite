use anyhow::{anyhow, Result};
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
use liquid::{ParserBuilder, object};
use orgish::{
    parse::{parse_n_pass, AstNode, HeaderRouting, stringify_bet},
    treewalk::ast_to_html_string,
};
use std::str::FromStr;
use typed_html::{dom::DOMTree, html, text, unsafe_text};

use crate::ARGS;

static STATIC_DIR: CompDir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../data/static");

lazy_static! {
    static ref INDEX_URI: Uri = Uri::from_str("/index").unwrap();
    static ref CONTENT_DIR: Dir = Dir::open_ambient_dir(&ARGS.content_path, ambient_authority()).unwrap();
}

pub async fn fallback_handler<B>(req: Request<B>) -> Result<Response> {
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
                let mut resp = (
                    StatusCode::OK,
                    f.contents_utf8()
                        .ok_or(anyhow!("file to be utf-8"))?
                        .to_owned(),
                )
                    .into_response();

                resp.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap()); // TODO fix result mess so this can be ?Try

                resp
            }
        })
    } else {
        let parser = ParserBuilder::with_stdlib().build()?;
        let liquid_index = CONTENT_DIR.read_to_string("index.liquid")?;
        let template = parser.parse(&liquid_index)?;
        let globals = object!({
             "req_path": format!("{}", uri)
         });

         Ok(Html(template.render(&globals)?.to_string()).into_response())
    }
}