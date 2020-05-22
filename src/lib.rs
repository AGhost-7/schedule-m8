#[macro_use]
extern crate log;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

extern crate sled;
extern crate priority_queue;
extern crate rmp_serde;

extern crate hyper;
extern crate futures;
extern crate bytes;
extern crate cron;

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{
    Server,
    service::{make_service_fn, service_fn}
};

use futures::channel::oneshot;

mod error;

mod keyspace;

pub mod schema;

mod scheduler;
use scheduler::Scheduler;

mod store;
use crate::store::Store;

mod api;
use crate::api::handle_request;

mod shard;
mod cluster;
use crate::cluster::Cluster;

pub struct ScheduleM8 {
    scheduler: Scheduler,
    close_sender: oneshot::Sender<()>,
    closed_receiver: oneshot::Receiver<()>
}

type GenericError = Box<dyn std::error::Error + Send + Sync>;

impl ScheduleM8 {
    pub async fn start(bind: String, db_path: String) -> ScheduleM8 {
        info!("Opening store at location: {}", db_path);
        let tree = sled::open(&db_path).expect("Failed to open database");
        let store = Arc::new(Store::new(tree));
        let cluster = Arc::new(Cluster::start(store.clone()).await);
        let scheduler = Scheduler::start(store.clone());

        let address: SocketAddr = bind.parse().unwrap();

        let make_svc = make_service_fn(move |_| {
            let service_cluster = cluster.clone();
            async {
                Ok::<_, GenericError>(service_fn(move |req| {
                    handle_request(service_cluster.clone(), req)
                }))
            }
        });

        let server: Server<_,_> = Server::bind(&address).serve(make_svc);
        let (close_sender, close_receiver) = oneshot::channel::<()>();
        let (closed_sender, closed_receiver) = oneshot::channel::<()>();

        tokio::spawn(async {
            server
                .with_graceful_shutdown(async {
                    close_receiver.await.ok();
                })
            .await
                .unwrap();
            closed_sender.send(()).unwrap();
        });

        ScheduleM8 {
            scheduler,
            close_sender,
            closed_receiver
        }
    }

    pub fn stop(self) {
        self.scheduler.stop();
        self.close_sender.send(()).unwrap();
    }

    pub async fn forever(self) {
        self.closed_receiver.await.unwrap();
    }
}
