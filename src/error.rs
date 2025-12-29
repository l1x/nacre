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
        let (status, user_message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, format!("Not found: {}", msg)),
            AppError::Beads(BeadsError::NotFound(msg)) => {
                (StatusCode::NOT_FOUND, format!("Not found: {}", msg))
            }
            AppError::Beads(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal error occurred".to_string(),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, format!("Bad request: {}", msg)),
        };

        tracing::error!("{}", self);
        (status, user_message).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
