/// A newtype wrapper around `anyhow` so we can implement axum's IntoResponse for it.
///
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use thiserror::Error as ThisError;
use tracing::error;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Anyhow(anyhow::Error::from(err))
    }
}

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
