// This contains all of the top level errors in the application

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum AppError {
    ValidationError,
    // this is likely a bug...
    UnexpectedError,
    // internal shard rpc calls failed
    NodeUnreachable,
    RpcDeserializationError,
    // fallback error if unable to parse the grpc status
    UnexpectedRpcError(String)
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        match &*self {
            AppError::ValidationError => write!(formatter, "ValidationError"),
            AppError::UnexpectedError => write!(formatter, "UnexpectedError"),
            AppError::NodeUnreachable => write!(formatter, "NodeUnreachable"),
            AppError::RpcDeserializationError => write!(formatter, "RpcDeserializationError"),
            AppError::UnexpectedRpcError(message) => write!(formatter, "UnexpectedRpcError - {}", message)
        }
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        match &*self {
            AppError::ValidationError => "ValidationError",
            AppError::UnexpectedError => "UnexpectedError",
            AppError::NodeUnreachable => "NodeUnreachable",
            AppError::RpcDeserializationError => "RpcDeserializationError",
            AppError::UnexpectedRpcError(_) => "UnexpectedRpcError"
        }
    }
}

impl From<SerdeError> for AppError {
    fn from(_error: SerdeError) -> AppError {
        AppError::ValidationError
    }
}
