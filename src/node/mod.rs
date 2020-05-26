mod grpc;
mod convert;
pub mod client;
pub mod server;

#[cfg(test)]
mod test_node {
    use crate::schema::Job;
    use crate::cluster::Cluster;
    use crate::store::Store;
    use crate::node::server::NodeServer;
    use crate::node::client::NodeClient;
    use std::sync::Arc;
    use sled::Db;
    use std::time::{UNIX_EPOCH, SystemTime, Duration};

    fn random_host() -> String {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        let port = rng.gen_range(7001, 10000);
        "127.0.0.1:".to_owned() + &port.to_string()
    }

    #[tokio::test]
    async fn node_push() {
use std::env;
use env_logger::Env;
        env_logger::from_env(
            Env::default().default_filter_or("info")
        ).init();
        let db = Db::open(".test/node").unwrap();
        let store = Arc::new(Store::new(db));
        let cluster = Arc::new(Cluster::start(store.clone()).await);
        let host = random_host();
        let server = NodeServer::start(host.parse().unwrap(), cluster).await;
        let client_url = String::from("http://") + &host;
        let client = NodeClient::connect(&client_url).await.unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        client.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(100),
            id: "yolo".to_owned(),
            schedule: None
        }).await.unwrap();

        store.next().unwrap();
    }
}
