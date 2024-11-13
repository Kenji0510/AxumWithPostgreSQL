use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::error;


#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database Error")]
    DatabaseError,
    // HTTP errors
    #[error("Not Found")]
    NotFound,
    #[error("Bad Request")]
    BadRequest,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Conflict")]
    Conflict,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to action on database".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, "Bad request".to_string()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, "Note with that title already exists".to_string()),
        };

        let error_response = ErrorResponse {
            status: "Error".to_string(),
            error_message: Some(message),
        };

        (status, Json(error_response)).into_response()
    }
}