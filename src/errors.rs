use actix_web::{http::StatusCode, HttpResponse};
use base64::DecodeError as Base64DecodeError;
use hyper::http::uri::InvalidUri;
use serde::Serialize;
use serde_json::Error as SerdeJsonError;
use std::sync::PoisonError;
use thiserror::Error;

/// Application level errors for the Bio-DID-Seq service
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IPFS error: {0}")]
    IPFSError(#[from] ipfs_api::Error),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Service error: {0}")]
    ServiceError(String),

    #[error("Serialization error")]
    SerializationError,

    #[error("Deserialization error")]
    DeserializationError,

    #[error("File error: {0}")]
    FileError(String),

    #[error("HTTP request error: {0}")]
    RequestError(String),

    #[error("Dataverse API error: {0}")]
    DataverseApiError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl actix_web::error::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::IPFSError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::ServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SerializationError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DeserializationError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::FileError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RequestError(_) => StatusCode::BAD_REQUEST,
            AppError::DataverseApiError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.status_code().as_str().to_string(),
            message: self.to_string(),
        })
    }
}

// Explicit conversions from common error types
impl From<mysql_async::Error> for AppError {
    fn from(error: mysql_async::Error) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}

impl From<mysql_async::UrlError> for AppError {
    fn from(error: mysql_async::UrlError) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(_error: serde_json::Error) -> Self {
        AppError::DeserializationError
    }
}

// Handle mutex poisoning
impl<T> From<PoisonError<T>> for AppError {
    fn from(err: PoisonError<T>) -> Self {
        AppError::ServiceError(format!("Mutex lock failed: {}", err))
    }
}

impl From<Base64DecodeError> for AppError {
    fn from(err: Base64DecodeError) -> Self {
        AppError::AuthError(format!("Base64 decoding error: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ExternalServiceError(format!("HTTP request error: {}", err))
    }
}

/// Possible errors that can occur in the service
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] mysql_async::Error),
    #[error("IPFS error: {0}")]
    Ipfs(#[from] ipfs_api::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] InvalidUri),
    #[error("URL parsing error: {0}")]
    UrlError(#[from] mysql_async::UrlError),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limit exceeded")]
    RateLimit,
}

impl actix_web::error::ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::InvalidInput(_) | ServiceError::Validation(_) => StatusCode::BAD_REQUEST,
            ServiceError::Auth(_) => StatusCode::UNAUTHORIZED,
            ServiceError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.status_code().as_str().to_string(),
            message: self.to_string(),
        })
    }
}

// From implementations for completeness
impl From<bcrypt::BcryptError> for ServiceError {
    fn from(err: bcrypt::BcryptError) -> Self {
        ServiceError::Internal(format!("Password hashing error: {}", err))
    }
}

impl From<validator::ValidationErrors> for ServiceError {
    fn from(err: validator::ValidationErrors) -> Self {
        ServiceError::Validation(err.to_string())
    }
}

impl From<actix_multipart::MultipartError> for ServiceError {
    fn from(err: actix_multipart::MultipartError) -> Self {
        ServiceError::Internal(format!("Multipart error: {}", err))
    }
}

// Handle mutex poisoning
impl<T> From<PoisonError<T>> for ServiceError {
    fn from(err: PoisonError<T>) -> Self {
        ServiceError::Internal(format!("Mutex lock failed: {}", err))
    }
}

impl From<SerdeJsonError> for ServiceError {
    fn from(err: SerdeJsonError) -> Self {
        ServiceError::Internal(format!("Serialization error: {}", err))
    }
}

impl From<Base64DecodeError> for ServiceError {
    fn from(err: Base64DecodeError) -> Self {
        ServiceError::Auth(format!("Base64 decoding error: {}", err))
    }
}

/// Error response for API endpoints
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}
