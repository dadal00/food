use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Malformed payload")]
    MalformedPayload,

    #[error("Internal error: {0}")]
    InternalError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::MalformedPayload { .. } => StatusCode::BAD_REQUEST,
            AppError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
