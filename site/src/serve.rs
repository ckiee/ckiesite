use anyhow::{anyhow, Result};
use axum::{
    http::HeaderValue,
    response::{Html, IntoResponse, Response},
};
use cap_std::{ambient_authority, fs::Dir};
use hyper::{
    header::{CONTENT_TYPE}, Request, StatusCode, Uri,
};
use include_dir::{include_dir, Dir as CompDir};
use lazy_static::lazy_static;
use liquid::{object, ParserBuilder};
use orgish::{
    parse::{parse_n_pass, stringify_bet, AstNode, Route},
    treewalk::ast_to_html_string,
};
use std::str::FromStr;


use crate::ARGS;

static STATIC_DIR: CompDir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../data/static");

lazy_static! {
    static ref INDEX_URI: Uri = Uri::from_str("/index").unwrap();
    static ref CONTENT_DIR: Dir =
        Dir::open_ambient_dir(&ARGS.content_path, ambient_authority()).unwrap();
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
        let liquid_parser = ParserBuilder::with_stdlib().build()?;
        let org_file = CONTENT_DIR.read_to_string("index.org")?;
        let ast = parse_n_pass(&org_file)?;

        for node in ast {
            match &node.inner {
                // we trim off the first byte since it's probably `/` and that doesn't match the hashmap keys
                // sure do hope it's not some unicode scalar that will do really weird things and make us panic
                AstNode::Heading {
                    routing: Some(Route::Page(pg)),
                    title,
                    children,
                    ..
                } if pg == &uri.path()[1..] => {
                    let html = ast_to_html_string(children, None)?; // TODO actually use Some(..RenderGroup
                    let _text_title = format!("{:?}", title);
                    let liquid_page = CONTENT_DIR.read_to_string("page.liquid")?;
                    let template = liquid_parser.parse(&liquid_page)?;
                    let globals = object!({
                        "req_path": format!("{}", uri),
                        "html": html,
                        "title": stringify_bet(title)?
                    });

                    return Ok(Html(template.render(&globals)?).into_response());
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
