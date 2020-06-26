use std::sync::Arc;
use std::net::SocketAddr;
use crate::cluster::Cluster;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use futures::channel::oneshot;
use tokio::time::interval;
use std::time::Duration;

use super::grpc::node_server::{Node, NodeServer as GrpcNodeServer};
use super::grpc;
use crate::schema::Job;
use super::convert::*;

pub struct NodeService {
    cluster: Arc<Cluster>
}

#[tonic::async_trait]
impl Node for NodeService {
    async fn push(&self, request: Request<grpc::Job>) -> Result<Response<grpc::Job>, Status> {
        let job = Job::try_from(request.into_inner())?;
        self.cluster.push(job.clone()).await?;
        Ok(Response::new(grpc::Job::from(job)))
    }

    async fn remove(&self, request: Request<grpc::Id>) -> Result<Response<grpc::RemoveResponse>, Status> {
        let id = request.into_inner().id;
        let job = self.cluster.remove(&id).await?.map(grpc::Job::from);
        Ok(Response::new(grpc::RemoveResponse{ job: job }))
    }

    async fn clear(&self, _request: Request<grpc::Empty>) -> Result<Response<grpc::Empty>, Status> {
        self.cluster.clear().await?;
        Ok(Response::new(grpc::Empty { }))
    }
}

pub struct NodeServer {
    close_sender: oneshot::Sender<()>,
    closed_receiver: oneshot::Receiver<()>
}

impl NodeServer {
    pub async fn start(addr: SocketAddr, cluster: Arc<Cluster>) -> NodeServer {
        let service = NodeService {
            cluster
        };
        let (close_sender, close_receiver) = oneshot::channel::<()>();
        let (closed_sender, closed_receiver) = oneshot::channel::<()>();

        let close_future = async {
            close_receiver.await.unwrap();
        };
        let serve = Server::builder()
            .add_service(GrpcNodeServer::new(service))
            .serve_with_shutdown(addr, close_future);

        tokio::spawn(async move {
            serve.await.unwrap();
            closed_sender.send(()).unwrap();
        });

        // for some reason, this is the only way to make sure when the `start`
        // future completes that the server is listening on the port...
        interval(Duration::from_millis(100)).tick().await;

        NodeServer {
            close_sender,
            closed_receiver
        }
    }

    pub fn stop(self) {
        self.close_sender.send(()).unwrap();
    }

    pub async fn forever(self) {
        self.closed_receiver.await.unwrap();
    }
}
