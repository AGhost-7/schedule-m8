use sled::Db;
use rmp_serde::Serializer;
use priority_queue::PriorityQueue;
use crate::schema::Job;
use serde::Serialize;
use std::time::{UNIX_EPOCH, Duration, SystemTime};
use std::sync::Mutex;

use crate::keyspace::KEYSPACE_QUEUE;

pub struct Store {
    queue: Mutex<PriorityQueue<String, Duration>>,
    tree: Db
}

impl Store {
    pub fn new(tree: Db) -> Self {
        let mut queue = PriorityQueue::new();
        for serialized in tree.scan_prefix(KEYSPACE_QUEUE).values() {
            let item: Job = rmp_serde::decode::from_slice(
                &serialized.expect("Failed to extract from store")
            ).expect("Failed to deserialize from store");
            let priority = item.timestamp.clone();
            queue.push(item.id.clone(), priority);
        }
        Store {
            queue: Mutex::new(queue),
            tree: tree
        }
    }

    fn db_key(id: &str) -> Box<[u8]> {
        let bytes = id.as_bytes();
        let mut key: Vec<u8> = Vec::with_capacity(KEYSPACE_QUEUE.len() + bytes.len());
        key.extend(KEYSPACE_QUEUE.iter());
        key.extend(bytes);
        key.into_boxed_slice()
    }

    pub fn next(&self) -> Option<Job> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error getting system time");
        let mut queue = self.queue.lock().expect("Failed to acquire lock");

        let has_next = queue
            .peek()
            .map(|(_, timestamp)| timestamp.lt(&now) || timestamp.eq(&now))
            .unwrap_or(false);

        if has_next {
            queue.pop().map(|(uuid, _)| {
                let bytes = self.tree.remove(Store::db_key(&uuid))
                    .expect("Failed to remove item from tree")
                    .expect("Item in queue does not exist in persistence layer");

                let item: Job = rmp_serde::decode::from_slice(&bytes)
                    .expect("Failed to deserialize from store");

                self.tree.remove(&uuid.as_bytes()).expect("Failed to remove item from tree");
                item
            })
        } else {
            None
        }
    }

    pub fn push(&self, item: Job) {
        let priority = item.timestamp.clone();
        let mut buffer = Vec::new();
        item
            .serialize(&mut Serializer::new(&mut buffer))
            .expect("Failed to serialize callback");
        self
            .queue
            .lock()
            .expect("Failed to acquire lock")
            .push(item.id.clone(), priority);
        self.tree.insert(Store::db_key(&item.id), buffer).unwrap();
    }

    pub fn remove(&self, id: &str) -> Option<Job> {
        let serialized = self
            .tree
            .remove(Store::db_key(id))
            .expect("Failed to remove callback from storage");

        serialized.map(|data| {
            let item: Job = rmp_serde::decode::from_slice(&data).unwrap();
            let mut queue = self.queue.lock().expect("Failed to acquire lock");
            queue.change_priority(&item.id, Duration::new(std::u64::MAX, 0));
            queue.pop();
            item
        })
    }

    pub fn clear(&self) {
        self.tree.clear().expect("Failed to clear storage");
        self.queue.lock().expect("Failed to acquire lock").clear();
    }
}

#[cfg(test)]
mod test {
    use std::time::{UNIX_EPOCH, SystemTime, Duration};
    use uuid::Uuid;
    use crate::schema::Job;
    use std::thread;
    use std::sync::Arc;
    use super::*;

    #[test]
    fn duplicates() {
        let tree = sled::open(".test/duplicates").expect("Failed to open store");
        let store = Store::new(tree);
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        store.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(200),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });
        store.push(Job {
            method: "POST".to_owned(),
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(100),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });
        store.push(Job {
            method: "POST".to_owned(),
            url: "3".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(200),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });

        assert_eq!(store.next().unwrap().url, "2");
        assert_eq!(store.next().unwrap().url, "3");
        assert_eq!(store.next().unwrap().url, "1");
    }

    #[test]
    fn remove() {
        let tree = sled::open(".test/remove").expect("Failed to open store");
        let store = Store::new(tree);
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let id = Uuid::new_v4().to_string();
        store.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(100),
            id: id.clone(),
            schedule: None
        });

        store.push(Job {
            method: "POST".to_owned(),
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now - Duration::from_millis(100),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });
        store.remove(&id);
        assert_eq!(store.next().unwrap().url, "2");
        assert!(store.next().is_none());
    }

    #[test]
    fn resume() {
        {
            let tree = sled::open(".test/resume").expect("Failed to open store");
            let store = Store::new(tree);
            store.clear();
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            store.push(Job {
                method: "POST".to_owned(),
                url: "1".to_owned(),
                body: "{}".to_owned(),
                timestamp: now - Duration::from_millis(100),
                id: Uuid::new_v4().to_string(),
                schedule: None
            });
        }

        let tree = sled::open(".test/resume").expect("Failed to open store");
        let store = Store::new(tree);
        assert_eq!(store.next().unwrap().url, "1");
        assert_eq!(store.next(), None);
    }

    #[test]
    fn not_has_next() {
        let tree = sled::open(".test/not_has_next").expect("Failed to open store");
        let store = Store::new(tree);
        store.clear();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        store.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });

        assert_eq!(store.next(), None);
    }


    #[test]
    fn multi_threaded() {
        let tree = sled::open(".test/multi_threaded").unwrap();
        let store = Store::new(tree);
        let store_arc = Arc::new(store);
        let clone = Arc::clone(&store_arc);
        let child = thread::spawn(move || {
            clone.next();
        });
        child.join().unwrap();
    }
}
