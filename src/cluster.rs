use crate::store::Store;
use std::sync::Arc;
use crate::shard::Shard;
use crate::schema::Job;
use crate::error::AppError;
use std::collections::hash_map::DefaultHasher;
use std::hash::*;
use tokio::sync::RwLock;

pub struct Cluster {
    shards: RwLock<Vec<Shard>>
}

impl Cluster {
    pub async fn start(store: Arc<Store>) -> Cluster {
        let mut shards: Vec<Shard> = Vec::with_capacity(128);
        for _ in 0..127 {
            shards.push(Shard::Local(store.clone()));
        }
        // hashing is not implemented to handle beyond that value.
        assert!(shards.len() < usize::MAX);
        Cluster {
            shards: RwLock::new(shards)
        }
    }

    fn shard<'a>(shards: &'a Vec<Shard>, id: &str) -> &'a Shard {
        let mut hasher = DefaultHasher::new();
        (*id).hash(&mut hasher);
        let hash = hasher.finish();
        let index = (hash % shards.len() as u64) as usize;
        shards.get(index).expect("Could not find shard at given id")
    }

    pub async fn push(&self, job: Job) -> Result<(), AppError> {
        let shards = self.shards.read().await;
        let shard = Cluster::shard(&shards, &job.id);

        shard.push(job).await
    }

    pub async fn remove(&self, id: &str) -> Result<Option<Job>, AppError> {
        let shards = self.shards.read().await;
        let shard = Cluster::shard(&shards, id);
        shard.remove(id).await
    }

    pub async fn clear(&self) -> Result<(), AppError> {
        for shard in self.shards.read().await.iter() {
            shard.clear().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;
    use std::time::{UNIX_EPOCH, SystemTime, Duration};
    use std::thread::{JoinHandle, spawn};

    fn random_job() -> Job {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_millis(10000);
        Job {
            method: "POST".to_owned(),
            url: "http://schedule-m8-test.local:1111".to_owned(),
            body: "{}".to_owned(),
            timestamp,
            id: Uuid::new_v4().to_string(),
            schedule: None
        }
    }

    //#[tokio::test]
    //async fn push_local() {
    //    tokio::fs::remove_dir_all(".test/push-local").await.unwrap();
    //    let tree = sled::open(".test/push-local").unwrap();
    //    let store = Arc::new(Store::new(tree));
    //    let cluster = Arc::new(Cluster::start(store).await);

    //    let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();
    //    for _ in 1..10 {
    //        let clone = cluster.clone();
    //        let handle = spawn(move || {
    //            for _ in 1..100 {
    //                clone.push(random_job()).await.unwrap();
    //            }
    //        });
    //        thread_handles.push(handle);
    //    }

    //    for handle in thread_handles {
    //        handle.join().unwrap();
    //    }
    //}
}
