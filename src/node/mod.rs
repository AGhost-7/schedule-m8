mod grpc;
pub mod client;
pub mod server;

#[cfg(test)]
mod test {
    use crate::cluster::Cluster;
    use crate::store::Store;
    use std::sync::Arc;
    use sled::Db;

    fn random_port() -> u16 {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        rng.gen_range(7001, 10000)
    }

    #[tokio::test]
    async fn server() {
        let db = Db::open(".test/node");
        let store = Arc::new(Store::new(db));
        let cluster = Cluster::start(store).await;
        let server = server::NodeServer::start(cluster).await;
    }
}
