use std::fmt;
#[derive(Debug)]
pub enum AppError {
    Database(rusqlite::Error),
    Pool(r2d2::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(String),
    Validation(String),
    Conflict(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "database error: {e}"),
            AppError::Pool(e) => write!(f, "connection pool error: {e}"),
            AppError::Io(e) => write!(f, "io error: {e}"),
            AppError::Json(e) => write!(f, "serialization error: {e}"),
            AppError::NotFound(what) => write!(f, "not found: {what}"),
            AppError::Validation(msg) => write!(f, "validation error: {msg}"),
            AppError::Conflict(msg) => write!(f, "conflict: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(ref err, ref msg) = e {
            if err.code == rusqlite::ErrorCode::ConstraintViolation {
                let detail = msg.clone().unwrap_or_else(|| "constraint violation".into());
                return AppError::Conflict(detail);
            }
        }
        AppError::Database(e)
    }
}

impl From<r2d2::Error> for AppError {
    fn from(e: r2d2::Error) -> Self {
        AppError::Pool(e)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
