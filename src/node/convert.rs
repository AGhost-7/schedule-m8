// type conversions between the grpc and internal types
use tonic::{ Status, Code };
use std::time::Duration;
use crate::schema::Job;
use crate::error::AppError;
use super::grpc;

use prost::Message;

pub use std::convert::TryFrom;

fn rpc_method(method: &str) -> i32 {
    match method {
        "GET" => 0,
        "HEAD" => 1,
        "POST" => 2,
        "PUT" => 3,
        "DELETE" => 4,
        "CONNECT" => 5,
        "OPTIONS" => 6,
        "TRACE" => 7,
        "PATCH" => 8,
        _ => -1
    }
}

fn internal_method(method: i32) -> String {
    let conversion = match method {
        0 => "GET",
        1 => "HEAD",
        2 => "POST",
        3 => "PUT",
        4 => "DELETE",
        5 => "CONNECT",
        6 => "OPTIONS",
        7 => "TRACE",
        8 => "PATCH",
        _ => ""
    };
    conversion.to_owned()
}

impl From <Status> for AppError {
    fn from(status: Status) -> AppError {
        match grpc::AppError::decode(status.details()) {
            Err(_) => {
                AppError::UnexpectedRpcError(status.message().to_owned())
            },
            Ok(decoded) => {
                match decoded.code {
                    1 => AppError::ValidationError,
                    2 => AppError::UnexpectedError,
                    3 => AppError::NodeUnreachable,
                    4 => AppError::RpcDeserializationError,
                    5 => {
                        error!("app_error - {}", decoded.message);
                        AppError::UnexpectedRpcError(decoded.message)
                    },
                    _ => {
                        error!("app_error - invalid rpc error response");
                        AppError::RpcDeserializationError
                    }
                }
            }
        }
    }
}

impl From <Job> for grpc::Job {
    fn from(job: Job) -> grpc::Job {
        grpc::Job {
            id: job.id,
            timestamp: job.timestamp.as_millis() as u64,
            method: rpc_method(&job.method),
            url: job.url,
            body: job.body,
            has_schedule: job.schedule.is_some(),
            schedule: job.schedule.unwrap_or("".to_owned())
        }
    }
}

impl TryFrom <grpc::Job> for Job {
    type Error = AppError;

    fn try_from(rpc_job: grpc::Job) -> Result<Job, AppError> {
        Ok(Job {
            id: rpc_job.id,
            timestamp: Duration::from_millis(rpc_job.timestamp),
            method: internal_method(rpc_job.method),
            url: rpc_job.url,
            body: rpc_job.body,
            schedule: match rpc_job.has_schedule {
                true => Some(rpc_job.schedule),
                false => None
            }
        })
    }
}

impl From<AppError> for Status {
    fn from(app_error: AppError) -> Status {
        let mut grpc_error = grpc::AppError::default();
        let message = app_error.to_string();

        let code = match app_error {
            AppError::ValidationError => {
                grpc_error.code = 1;
                Code::InvalidArgument
            },
            AppError::UnexpectedError => {
                grpc_error.code = 2;
                Code::Unknown
            },
            AppError::NodeUnreachable => {
                grpc_error.code = 3;
                Code::Unavailable
            },
            AppError::RpcDeserializationError => {
                grpc_error.code = 4;
                Code::InvalidArgument
            },
            AppError::UnexpectedRpcError(message) => {
                grpc_error.message = message.clone();
                grpc_error.code = 5;
                Code::Unknown
            }
        };
        let mut encoded = Vec::new();
        grpc_error.encode(&mut encoded)
            .expect("Failed to allocate buffer to write error");

        Status::with_details(code, message, bytes::Bytes::from(encoded))
    }
}
