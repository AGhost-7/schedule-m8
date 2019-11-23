// delete(id): log(n)
// push: log(n)
// pop: log(n)

use std::collections::{BinaryHeap, LinkedList};
use sled::Db;

pub struct Store <K, T> {
    heap: BinaryHeap<T>,
    tree: Db
}

struct HeapContainer<K, T> {
    id: K,
    items: LinkedList<T>
}

impl <K, T> Store <K, T> where T: Ord {
    pub fn open<A, B>(path: &str) -> Result<Self, sled::Error> where A: Ord {
        let tree = Db::open(path)?;
        Ok(
            Store {
                heap: BinaryHeap::new(),
                tree: tree
            }
        )
    }
    pub fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }
    pub fn insert(&mut self, key: T, item: T) {
        self.heap.push(item);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }
}

#[test]
fn inserting_duplicates() {
    let mut store = Store::open::<u32>(".test/data").expect("Failed to open store");

    store.push(1);
    store.push(2);
    store.push(1);
    assert_eq!(store.pop(), Some(1));
    assert_eq!(store.pop(), Some(1));
    assert_eq!(store.pop(), Some(2));
    assert_eq!(store.pop(), None);
}
