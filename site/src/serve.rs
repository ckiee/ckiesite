
use crate::result::{Result};
use anyhow::anyhow;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use hyper::{Request, StatusCode, Uri};
use axum::{
    response::{IntoResponse},
};
use std::str::FromStr;


static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../data/static");

lazy_static! {
    static ref INDEX_ORG_URI: Uri = Uri::from_str("/index.org").unwrap();
}

pub async fn fallback_handler<B>(req: Request<B>) -> Result<impl IntoResponse> {
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
        // Ok(match ORG_DOCS.get(&uri.path()[1..]) {
        //     None => (StatusCode::NOT_FOUND, "):".to_string()).into_response(),
        //     Some(f) => Html(f.render_page_html()?).into_response(),
        // })
        unimplemented!();
    }
}
