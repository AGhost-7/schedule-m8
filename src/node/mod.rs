mod grpc;
mod convert;
pub mod client;
pub mod server;

#[cfg(test)]
mod test {
    use crate::schema::Job;
    use crate::cluster::Cluster;
    use crate::store::Store;
    use crate::node::server::NodeServer;
    use crate::node::client::NodeClient;
    use std::sync::Arc;
    use sled::Db;
    use std::time::{UNIX_EPOCH, SystemTime, Duration};
    use uuid::Uuid;

    fn random_host() -> String {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        let port = rng.gen_range(7001, 10000);
        "127.0.0.1:".to_owned() + &port.to_string()
    }

    macro_rules! node_test {
        ($name:ident $test:expr) => {
            #[tokio::test]
            async fn $name() {
                let data_dir = ".test/".to_owned() + &Uuid::new_v4().to_string();
                let db = Db::open(data_dir.clone()).unwrap();
                let store = Arc::new(Store::new(db));
                store.clear();
                let cluster = Arc::new(Cluster::start(store.clone()).await);
                let host = random_host();
                let server = NodeServer::start(host.parse().unwrap(), cluster).await;
                let client_url = String::from("http://") + &host;
                let client = NodeClient::connect(&client_url).await.unwrap();

                $test

                server.stop();
                tokio::fs::remove_dir_all(&data_dir).await.expect("Error removing directory");
            }
        }
    }

    node_test!(push {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        client.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(100),
            id: "yolo".to_owned(),
            schedule: None
        }).await.unwrap();

        let job = store.next().unwrap();
        assert!(job.id == "yolo");
    });

    node_test!(remove {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let id = "test";
        client.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(1000),
            id: id.to_owned(),
            schedule: None
        }).await.unwrap();
        let job = client.remove(id).await.unwrap().unwrap();
        assert!(job.method == "POST");
        assert!(job.id == id);
        assert_eq!(store.remove(id), None);
    });

    node_test!(clear {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let id = "test";
        let job = Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(1000),
            id: id.to_owned(),
            schedule: None
        };

        client.push(job.clone()).await.unwrap();

        client.clear().await.unwrap();
        assert_eq!(store.remove(&job.id), None);
    });
}
