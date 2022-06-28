use anyhow::{anyhow, Result};
use axum::{
    extract::Query,
    http::HeaderValue,
    response::{Html, IntoResponse, Response},
};
use cap_std::{ambient_authority, fs::Dir};
use hyper::{header::CONTENT_TYPE, Request, StatusCode, Uri};
use include_dir::{include_dir, Dir as CompDir};
use lazy_static::lazy_static;
use liquid::{object, ParserBuilder};
use orgish::{
    parse::{parse_n_pass, stringify_bet, AstNode, OutputTo, Route},
    treewalk::{ast_to_html_string, bet_to_html_string},
};
use std::fmt::Write;
use std::str::FromStr;

use crate::ARGS;

lazy_static! {
    static ref INDEX_URI: Uri = Uri::from_str("/index").unwrap();
    static ref CONTENT_DIR: Dir =
        Dir::open_ambient_dir(&ARGS.content_path, ambient_authority()).unwrap();
    static ref STATIC_DIR: Dir =
        Dir::open_ambient_dir(&ARGS.static_path, ambient_authority()).unwrap();
}

enum OutputFormat {
    Html,
    Ast,
}

pub async fn fallback_handler<B>(req: Request<B>) -> Result<Response> {
    let uri = if req.uri() == "/" {
        &INDEX_URI
    } else {
        req.uri()
    };

    // Syntax !@*(#&@!(*#&))11
    let output_format = if let Some(q) = uri.query() {
        if q.contains("ast") {
            OutputFormat::Ast
        } else {
            OutputFormat::Html
        }
    } else {
        OutputFormat::Html
    };

    if uri.path().starts_with("/static") {
        let path = &uri.path()["/static/".len()..];
        let f = STATIC_DIR.read_to_string(path)?;
        let mime = mime_guess::from_path(path)
            .first()
            .ok_or_else(|| anyhow!("couldn't guess mimetype for static file"))?
            .to_string();
        let mut resp = (StatusCode::OK, f).into_response();

        resp.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_str(&mime).unwrap()); // TODO fix result mess so this can be ?Try

        Ok(resp)
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
                    let liquid_page = CONTENT_DIR.read_to_string("page.liquid")?;
                    let template = liquid_parser.parse(&liquid_page)?;
                    return match output_format {
                        OutputFormat::Html => {
                            let html_buffers = ast_to_html_string(children, OutputTo::Main)?;
                            let globals = object!({
                                "req_path": format!("{}", uri),
                                "html": html_buffers.main,
                                "nav_htmls": html_buffers.nav,
                                "nav_htmls_len": html_buffers.nav.len(),
                                "title": stringify_bet(&title)?,
                                "html_title": bet_to_html_string(&title)?,
                                "format": "html"
                            });

                            Ok(Html(template.render(&globals)?).into_response())
                        }

                        OutputFormat::Ast => {
                            let globals = object!({
                                "req_path": format!("{}", uri),
                                "html": format!("<pre>{children:#?}</pre>"),
                                "nav_htmls_len": 0,
                                "title": format!("AST dump of {}", stringify_bet(&title)?),
                                "html_title": format!(r#"<code>AST</code> dump of "{}""#, bet_to_html_string(&title)?),
                                "format": "ast"
                            });

                            Ok(Html(template.render(&globals)?).into_response())
                        }
                    };
                }
                _ => {}
            }
        }

        Ok((StatusCode::NOT_FOUND, "nothing here!".to_string()).into_response())
    }
}
