// This contains all of the top level errors in the application

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug)]
pub enum AppError {
    ValidationError
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match *self {
            AppError::ValidationError => write!(formatter, "ValidationError")
        }
    }
}
impl Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::ValidationError => "ValidationError"
        }
    }
}
