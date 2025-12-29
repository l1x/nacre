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

        if status == StatusCode::INTERNAL_SERVER_ERROR {
            (status, "Internal server error".to_string()).into_response()
        } else {
            (status, self.to_string()).into_response()
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_internal_error_is_generic() {
        let err = AppError::Beads(BeadsError::CommandError("sensitive info".to_string()));
        let response = err.into_response();
        
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        
        assert_eq!(body_str, "Internal server error");
    }

    #[tokio::test]
    async fn test_not_found_error_is_specific() {
        let err = AppError::NotFound("Issue 123".to_string());
        let response = err.into_response();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        
        assert_eq!(body_str, "Issue not found: Issue 123");
    }
}
