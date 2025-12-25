use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::beads::BeadsError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Issue not found: {0}")]
    NotFound(String),

    #[error("Beads error: {0}")]
    Beads(#[from] BeadsError),

    #[error("Invalid request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Beads(BeadsError::NotFound(_)) => StatusCode::NOT_FOUND,
            AppError::Beads(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        };

        tracing::error!("{}", self);
        (status, self.to_string()).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
