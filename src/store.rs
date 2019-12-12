use sled::Db;
use rmp_serde::Serializer;
use priority_queue::PriorityQueue;
use crate::schema::Job;
use serde::Serialize;
use std::time::{UNIX_EPOCH, Duration, SystemTime};

pub struct Store {
    queue: PriorityQueue<String, Duration>,
    tree: Db
}

impl Store {
    pub fn open(path: &str) -> Result<Self, sled::Error> {
        let tree = Db::open(path)?;
        let mut queue = PriorityQueue::new();
        for serialized in tree.iter().values() {
            let item: Job = rmp_serde::decode::from_slice(
                &serialized.expect("Failed to extract from store")
            ).expect("Failed to deserialize from store");
            let priority = item.timestamp.clone();
            queue.push(item.id.clone(), priority);
        }
        Ok(
            Store {
                queue: queue,
                tree: tree
            }
        )
    }

    pub fn next(&mut self) -> Option<Job> {
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

    pub fn pop(&mut self) -> Option<Job> {
        self.queue.pop().map(|(uuid, _)| {
            let bytes = self.tree.remove(&uuid.as_bytes())
                .expect("Failed to remove item from tree")
                .expect("Item in queue does not exist in persistence layer");

            let item: Job = rmp_serde::decode::from_slice(&bytes)
                .expect("Failed to deserialize from store");

            self.tree.remove(&uuid.as_bytes()).expect("Failed to remove item from tree");
            item
        })
    }

    pub fn push(&mut self, item: Job) {
        let priority = item.timestamp.clone();
        let id_bytes = item.id.as_bytes();
        let mut buffer = Vec::new();
        item
            .serialize(&mut Serializer::new(&mut buffer))
            .expect("Failed to serialize callback");
        self.queue.push(item.id.clone(), priority);
        self.tree.insert(id_bytes, buffer).unwrap();
    }

    pub fn remove(&mut self, id: &str) -> Option<Job> {
        let serialized = self
            .tree
            .remove(id.as_bytes())
            .expect("Failed to remove callback from storage");

        serialized.map(|data| {
            let item: Job = rmp_serde::decode::from_slice(&data).unwrap();
            self.queue.change_priority(&item.id, Duration::new(std::u64::MAX, 0));
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
    use crate::schema::Job;
    use super::*;

    #[test]
    fn duplicates() {
        let mut store = Store::open(".test/duplicates").expect("Failed to open store");
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
        store.push(Job {
            method: "POST".to_owned(),
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(200),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });
        store.push(Job {
            method: "POST".to_owned(),
            url: "3".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            id: Uuid::new_v4().to_string(),
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
        let id = Uuid::new_v4().to_string();
        store.push(Job {
            method: "POST".to_owned(),
            url: "1".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            id: id.clone(),
            schedule: None
        });

        store.push(Job {
            method: "POST".to_owned(),
            url: "2".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            id: Uuid::new_v4().to_string(),
            schedule: None
        });
        store.remove(&id);
        assert_eq!(store.pop().unwrap().url, "2");
        assert!(store.pop().is_none());
    }

    #[test]
    fn resume() {
        {
            let mut store = Store::open(".test/resume").expect("Failed to open store");
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
        }

        let mut store = Store::open(".test/resume").expect("Failed to open store");
        assert_eq!(store.pop().unwrap().url, "1");
        assert_eq!(store.pop(), None);
    }
}
