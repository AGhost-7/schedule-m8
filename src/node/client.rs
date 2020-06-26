use crate::error::AppError;
use crate::schema::Job;
use tonic::transport::Channel;

use super::convert::*;

use super::grpc::node_client::NodeClient as RpcClient;

use super::grpc;

pub struct NodeClient {
    rpc_client: RpcClient<Channel>
}

impl NodeClient {
    pub async fn connect(host: &str) -> Result<NodeClient, AppError> {
        let rpc_client = RpcClient::connect(host.to_owned())
            .await
            .map_err(|err| {
                error!("\n\n\nERROR RpcClient::connect err - {}", err);
                AppError::NodeUnreachable
            })?;
        Ok(NodeClient {
            rpc_client
        })
    }

    pub async fn push(&self, job: Job) -> Result<(), AppError> {
        let mut rpc_client = self.rpc_client.clone();
        rpc_client
            .push(grpc::Job::from(job))
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn remove(&self, id: &str) -> Result<Option<Job>, AppError> {
        let mut rpc_client = self.rpc_client.clone();

        let result = rpc_client
            .remove(grpc::Id { id: id.to_owned() })
            .await
            .map_err(AppError::from)?
            .into_inner();

        match result.job {
            None => Ok(None),
            Some(rpc_job) => Ok(Some(Job::try_from(rpc_job)?))
        }
    }

    pub async fn clear(&self) -> Result<(), AppError> {
        let mut rpc_client = self.rpc_client.clone();
        rpc_client
            .clear(grpc:: Empty { })
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}
