/// A newtype wrapper around `anyhow` so we can implement axum's IntoResponse for it.
use thiserror::Error;
use tracing::{error};
use axum::{
    response::{IntoResponse, Response},
};
use hyper::{StatusCode};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<A> = std::result::Result<A, Error>;

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
