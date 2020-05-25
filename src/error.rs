// This contains all of the top level errors in the application

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum AppError {
    ValidationError = 1,
    UnexpectedError = 2,
    NodeUnreachable = 3,
    InvalidNodeResponse = 4
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match *self {
            AppError::ValidationError => write!(formatter, "ValidationError"),
            AppError::UnexpectedError => write!(formatter, "UnexpectedError"),
            AppError::NodeUnreachable => write!(formatter, "NodeUnreachable"),
            AppError::InvalidNodeResponse => write!(formatter, "InvalidNodeResponse")
        }
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::ValidationError => "ValidationError",
            AppError::UnexpectedError => "UnexpectedError",
            // internal shard rpc calls failed
            AppError::NodeUnreachable => "NodeUnreachable",
            // this means the other shards are responding, but most likely
            // a deserialization error occurred
            AppError::InvalidNodeResponse => "InvalidNodeResponse"
        }
    }
}

impl From<SerdeError> for AppError {
    fn from(_error: SerdeError) -> AppError {
        AppError::ValidationError
    }
}
