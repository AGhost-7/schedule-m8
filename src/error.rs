// This contains all of the top level errors in the application

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum AppError {
    ValidationError,
    UnexpectedError
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match *self {
            AppError::ValidationError => write!(formatter, "ValidationError"),
            AppError::UnexpectedError => write!(formatter, "UnexpectedError")
        }
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::ValidationError => "ValidationError",
            AppError::UnexpectedError => "UnexpectedError"
                
        }
    }
}

impl From<SerdeError> for AppError {
    fn from(_error: SerdeError) -> AppError {
        AppError::ValidationError
    }
}
