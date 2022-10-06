use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use config::ConfigError;
use std::borrow::Cow;
use tokio::task::JoinError;
use zip::result::ZipError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Join error: {0}")]
    Join(#[from] JoinError),
    #[error("Zip error: {0}")]
    Zip(#[from] ZipError),
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("{0}")]
    Custom(Cow<'static, str>),
}

impl Error {
    pub fn custom<T, V: Into<Cow<'static, str>>>(value: V) -> Result<T> {
        Err(Self::Custom(value.into()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = self.to_string();

        (StatusCode::BAD_REQUEST, body).into_response()
    }
}
