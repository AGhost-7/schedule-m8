use crate::error::AppError;
use crate::store::Store;
use crate::schema::Job;
use crate::node::client::NodeClient;
use std::sync::Arc;

pub enum Shard {
    Local(Arc<Store>),
    Remote(Arc<NodeClient>),
    // Since we don't know where the document might be at since its in the
    // process of migrating them, certain actions need to call both of the
    // store and remote. For example, if you want to delete a record, the
    // record might still be in the store so you need to check there as well.
    Migrating(Arc<Store>, Arc<NodeClient>)
}

impl Shard {
    pub async fn push(&self, job: Job) -> Result<(), AppError> {
        match self {
            Shard::Local(store) => Ok(store.push(job)),
            Shard::Remote(client) => client.push(job).await,
            Shard::Migrating(store, client) => client.push(job).await
        }
    }
    pub async fn remove(&self, id: &str) -> Result<Option<Job>, AppError> {
        match self {
            Shard::Local(store) => Ok(store.remove(id)),
            Shard::Remote(client) => client.remove(id).await,
            Shard::Migrating(store, client) => {
                let local_result = store.remove(id);
                let remote_result = client.remove(id).await?;
                Ok(local_result.or(remote_result))
            }
        }
    }
    pub async fn clear(&self) -> Result<(), AppError> {
        match self {
            Shard::Local(store) => Ok(store.clear()),
            Shard::Remote(client) => client.clear().await,
            _ => Ok(()) // no need to handle this invariant
        }
    }
}
