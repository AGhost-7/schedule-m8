use crate::error::AppError;
use crate::schema::Job;
use tonic::transport::Channel;
use std::time::Duration;

use super::grpc::node_client::NodeClient as RpcClient;

use super::grpc;

pub struct NodeClient {
    rpc_client: RpcClient<Channel>
}

impl NodeClient {
    pub async fn connect(host: &str) -> Result<NodeClient, AppError> {
        let rpc_client = RpcClient::connect(host.to_owned())
            .await
            .map_err(|_| AppError::NodeUnreachable)?;
        Ok(NodeClient {
            rpc_client
        })
    }

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

    fn app_error(rpc_error: grpc::AppError) -> AppError {
        match rpc_error.code {
            1 => AppError::ValidationError,
            2 => AppError::UnexpectedError,
            3 => AppError::NodeUnreachable,
            4 => AppError::InvalidNodeResponse,
            _ => {
                error!("app_error - invalid rpc error response");
                AppError::InvalidNodeResponse
            }
        }
    }

    pub async fn push(&self, job: Job) -> Result<(), AppError> {
        let mut rpc_client = self.rpc_client.clone();
        let result = rpc_client
            .push(grpc::Job {
                id: job.id,
                timestamp: job.timestamp.as_millis() as u64,
                method: NodeClient::rpc_method(&job.method),
                url: job.url,
                body: job.body,
                has_schedule: job.schedule.is_some(),
                schedule: job.schedule.unwrap_or("".to_owned())
            })
            .await
            .map_err(|_| AppError::NodeUnreachable)?
            .into_inner();

        match result.result {
            Some(grpc::push_response::Result::Ok(_)) => {
                Ok(())
            },
            Some(grpc::push_response::Result::Err(err)) => {
                Err(NodeClient::app_error(err))
            },
            None => {
                Err(AppError::InvalidNodeResponse)
            }
        }
    }

    pub async fn remove(&self, id: &str) -> Result<Option<Job>, AppError> {
        let mut rpc_client = self.rpc_client.clone();

        let result = rpc_client
            .remove(grpc::Id { id: id.to_owned() })
            .await
            .map_err(|_| AppError::NodeUnreachable)?
            .into_inner();

        match result.result {
            Some(grpc::remove_response::Result::Ok(rpc_job)) => {
                Ok(Some(Job {
                    id: rpc_job.id,
                    timestamp: Duration::from_millis(rpc_job.timestamp),
                    method: NodeClient::internal_method(rpc_job.method),
                    url: rpc_job.url,
                    body: rpc_job.body,
                    schedule: match rpc_job.has_schedule {
                        true => Some(rpc_job.schedule),
                        false => None
                    }
                }))
            },
            Some(grpc::remove_response::Result::Err(err)) => {
                Err(NodeClient::app_error(err))
            },
            None => Err(AppError::InvalidNodeResponse)
        }
    }

    pub async fn clear(&self) -> Result<(), AppError> {
        unimplemented!();
    }
}
