use sled::Db;
use rmp_serde::Serializer;
use uuid::Uuid;
use priority_queue::PriorityQueue;
use std::time::Duration;
use crate::callback::Callback;
use serde::Serialize;

pub struct Store {
    queue: PriorityQueue<Callback, Duration>,
    tree: Db
}

impl Store {
    pub fn open(path: &str) -> Result<Self, sled::Error> {
        let tree = Db::open(path)?;
        let mut queue = PriorityQueue::new();
        for serialized in tree.iter().values() {
            let item: Callback = rmp_serde::decode::from_slice(
                &serialized.expect("Failed to extract from store")
            ).expect("Failed to deserialize from store");
            let priority = item.timestamp.clone();
            queue.push(item, priority);
        }
        Ok(
            Store {
                queue: queue,
                tree: tree
            }
        )
    }

    pub fn peek(&self) -> Option<&Callback> {
        self.queue.peek().map(|(item, _)| item)
    }

    pub fn pop(&mut self) -> Option<Callback> {
        self.queue.pop().map(|(item, _)| {
            self.tree.remove(&item.uuid.as_bytes()).expect("Failed to remove item from tree");
            item
        })
    }

    pub fn push(&mut self, item: Callback) {
        let priority = item.timestamp.clone();
        let uuid_bytes = item.uuid.as_bytes().clone();
        let mut buffer = Vec::new();
        item
            .serialize(&mut Serializer::new(&mut buffer))
            .expect("Failed to serialize callback");
        self.queue.push(item, priority);
        self.tree.insert(uuid_bytes, buffer).unwrap();
    }

    pub fn remove(&mut self, uuid: &Uuid) {
        let serialized = self
            .tree
            .remove(uuid.as_bytes())
            .expect("Failed to remove callback from storage");
        let item = rmp_serde::decode::from_slice(&serialized.unwrap()).unwrap();
        self.queue.change_priority(&item, Duration::new(std::u64::MAX, 0));
        self.queue.pop();
    }

    pub fn clear(&mut self) {
        self.tree.clear().expect("Failed to clear storage");
        self.queue.clear();
    }
}

#[cfg(test)]
mod test {
    use std::time::{UNIX_EPOCH, SystemTime, Duration};
    use serde::{Serialize, Deserialize};
    use uuid::Uuid;
    use crate::callback::Callback;
    use super::*;

    #[test]
    fn duplicates() {
        let mut store = Store::open(".test/data").expect("Failed to open store");
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        store.push(Callback {
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4()
        });
        store.push(Callback {
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(200),
            uuid: Uuid::new_v4()
        });
        store.push(Callback {
            url: "3".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4()
        });

        assert_eq!(store.pop().unwrap().url, "2");
        assert_eq!(store.pop().unwrap().url, "3");
        assert_eq!(store.pop().unwrap().url, "1");
    }

    #[test]
    fn remove() {
        let mut store = Store::open(".test/data").expect("Failed to open store");
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let uuid = Uuid::new_v4();
        store.push(Callback {
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: uuid.clone()
        });

        store.push(Callback {
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4()
        });
        assert!(store.peek().is_some());
        store.remove(&uuid);
        assert_eq!(store.pop().unwrap().url, "2");
        assert!(store.peek().is_none());
    }

    #[test]
    fn resume() {
        {
            let mut store = Store::open(".test/data").expect("Failed to open store");
            store.clear();
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            store.push(Callback {
                url: "1".to_owned(),
                body: "{}".to_owned(),
                timestamp: now + Duration::from_millis(100),
                uuid: Uuid::new_v4()
            });
        }

        let mut store = Store::open(".test/data").expect("Failed to open store");
        assert_eq!(store.pop().unwrap().url, "1");
        assert_eq!(store.pop(), None);
    }
}
