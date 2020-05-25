use crate::error::AppError;
use crate::store::Store;
use crate::schema::Job;
use std::sync::Arc;

pub enum Shard {
    Local(Arc<Store>)
    // Remote(Arc<NodeClient>)
    // Migrating
}

impl Shard {
    pub async fn push(&self, job: Job) -> Result<(), AppError> {
        match self {
            Shard::Local(store) => Ok(store.push(job))
        }
    }
    pub async fn remove(&self, id: &str) -> Result<Option<Job>, AppError> {
        match self {
            Shard::Local(store) => Ok(store.remove(id))
        }
    }
    pub async fn clear(&self) -> Result<(), AppError> {
        match self {
            Shard::Local(store) => Ok(store.clear())
        }
    }
}
