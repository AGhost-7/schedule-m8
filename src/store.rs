use sled::Db;
use rmp_serde::Serializer;
use uuid::Uuid;
use priority_queue::PriorityQueue;
use crate::callback::Callback;
use serde::Serialize;
use std::time::{UNIX_EPOCH, Duration, SystemTime};

pub struct Store {
    queue: PriorityQueue<Uuid, Duration>,
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
            queue.push(item.uuid.clone(), priority);
        }
        Ok(
            Store {
                queue: queue,
                tree: tree
            }
        )
    }

    pub fn next(&mut self) -> Option<Callback> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error getting system time");
        let has_next = self
            .queue
            .peek()
            .map(|(_, timestamp)| timestamp.lt(&now) || timestamp.eq(&now))
            .unwrap_or(false);

        if has_next {
            self.pop()
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<Callback> {
        self.queue.pop().map(|(uuid, _)| {
            let bytes = self.tree.remove(&uuid.as_bytes())
                .expect("Failed to remove item from tree")
                .expect("Item in queue does not exist in persistence layer");

            let item: Callback = rmp_serde::decode::from_slice(&bytes)
                .expect("Failed to deserialize from store");

            self.tree.remove(&uuid.as_bytes()).expect("Failed to remove item from tree");
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
        self.queue.push(item.uuid, priority);
        self.tree.insert(uuid_bytes, buffer).unwrap();
    }

    pub fn remove(&mut self, uuid: &Uuid) -> Option<Callback> {
        let serialized = self
            .tree
            .remove(uuid.as_bytes())
            .expect("Failed to remove callback from storage");

        serialized.map(|data| {
            let item: Callback = rmp_serde::decode::from_slice(&data).unwrap();
            self.queue.change_priority(&item.uuid, Duration::new(std::u64::MAX, 0));
            self.queue.pop();
            item
        })
    }

    pub fn clear(&mut self) {
        self.tree.clear().expect("Failed to clear storage");
        self.queue.clear();
    }
}

#[cfg(test)]
mod test {
    use std::time::{UNIX_EPOCH, SystemTime, Duration};
    use uuid::Uuid;
    use crate::callback::Callback;
    use super::*;

    #[test]
    fn duplicates() {
        let mut store = Store::open(".test/duplicates").expect("Failed to open store");
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        store.push(Callback {
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4(),
            schedule: None
        });
        store.push(Callback {
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(200),
            uuid: Uuid::new_v4(),
            schedule: None
        });
        store.push(Callback {
            url: "3".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4(),
            schedule: None
        });

        assert_eq!(store.pop().unwrap().url, "2");
        assert_eq!(store.pop().unwrap().url, "3");
        assert_eq!(store.pop().unwrap().url, "1");
    }

    #[test]
    fn remove() {
        let mut store = Store::open(".test/remove").expect("Failed to open store");
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let uuid = Uuid::new_v4();
        store.push(Callback {
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: uuid.clone(),
            schedule: None
        });

        store.push(Callback {
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4(),
            schedule: None
        });
        store.remove(&uuid);
        assert_eq!(store.pop().unwrap().url, "2");
        assert!(store.pop().is_none());
    }

    #[test]
    fn resume() {
        {
            let mut store = Store::open(".test/resume").expect("Failed to open store");
            store.clear();
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            store.push(Callback {
                url: "1".to_owned(),
                body: "{}".to_owned(),
                timestamp: now + Duration::from_millis(100),
                uuid: Uuid::new_v4(),
                schedule: None
            });
        }

        let mut store = Store::open(".test/resume").expect("Failed to open store");
        assert_eq!(store.pop().unwrap().url, "1");
        assert_eq!(store.pop(), None);
    }
}
