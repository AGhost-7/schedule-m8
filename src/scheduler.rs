
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use callback::Callback;
use std::time::{UNIX_EPOCH, Duration, SystemTime};
use crate::store::Store;
use std::sync::{Mutex,Arc};

pub struct Scheduler {
    rx: Receiver<Callback>,
    tx: Sender<Callback>,
    store: Arc<Mutex<Store>>
}

impl Scheduler {

    pub fn new(store: Arc<Mutex<Store>>, tx: Sender<Callback>, rx: Receiver<Callback>) -> Scheduler {
        Scheduler {
            rx: rx,
            tx: tx,
            store: store
        }
    }

    pub fn spawn(store: Arc<Mutex<Store>>) -> (Sender<Callback>, Receiver<Callback>) {
        let (tx, rx) = channel::<Callback>();
        let (trigger_tx, trigger_rx) = channel::<Callback>();
        let mut handler = Scheduler::new(store, trigger_tx, rx);
        thread::spawn(move|| {
            loop {
                handler.check_received();
                handler.check_to_send();
                thread::sleep(Duration::from_millis(1000));
            }
        });
        (tx, trigger_rx)
    }

    pub fn check_received(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(callback) => {
                    let mut store = self.store.lock().expect("Failed to acquire store lock due to poisoning");
                    store.push(callback);
                },
                Err(TryRecvError::Empty) => {
                    return;
                },
                Err(_) => {
                    panic!();
                }
            }
        }
    }

    pub fn check_to_send(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Error getting system time");
        let mut store = self.store.lock().expect("Failed to acquire store lock due to poisoning");
        loop {
            if let Some(item) = store.peek() {
                if item.timestamp.lt(&now) || item.timestamp.eq(&now) {
                    let elem = store.pop().unwrap();
                    self.tx.send(elem).expect("Scheduler channel disconnected.");
                    continue;
                }
            }
            break
        }
    }
}

#[test]
fn scheduling_callback() {
    use uuid::Uuid;

    let mut store = Store::open(".test/data").expect("Failed to open store");
    store.clear();
    let (tx, rx) = Scheduler::spawn(Arc::new(Mutex::new(store)));
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    tx.send(Callback {
        url: "1".to_owned(),
        body: "{}".to_owned(),
        timestamp: now + Duration::from_millis(100),
        uuid: Uuid::new_v4()
    }).unwrap();
    tx.send(Callback {
        url: "3".to_owned(),
        body: "{}".to_owned(),
        timestamp: now + Duration::from_millis(300),
        uuid: Uuid::new_v4()
    }).unwrap();
    tx.send(Callback {
        url: "2".to_owned(),
        body: "{}".to_owned(),
        timestamp: now + Duration::from_millis(200),
        uuid: Uuid::new_v4()
    }).unwrap();

    for i in ["1", "2", "3"].iter() {
        let res = rx.recv().unwrap();
        assert!(&res.url == i);
    }
}

#[test]
fn scheduling_duplicates() {

    use uuid::Uuid;

    let mut store = Store::open(".test/data").expect("Failed to open store");
    store.clear();
    let (tx, rx) = Scheduler::spawn(Arc::new(Mutex::new(store)));
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    tx.send(Callback {
        url: "1".to_owned(),
        body: "{}".to_owned(),
        timestamp: now + Duration::from_millis(100),
        uuid: Uuid::new_v4()
    }).unwrap();
    tx.send(Callback {
        url: "1".to_owned(),
        body: "{}".to_owned(),
        timestamp: now + Duration::from_millis(100),
        uuid: Uuid::new_v4()
    }).unwrap();
}

#[test]
fn shared_store() {
    use uuid::Uuid;

    let mut store = Store::open(".test/data").expect("Failed to open store");
    store.clear();
    let mutex = Arc::new(Mutex::new(store));
    let (tx, rx) = Scheduler::spawn(mutex.clone());
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    {
        mutex.lock().unwrap().push(Callback {
            url: "http://jokes.jonathan-boudreau.com".to_owned(),
            body: "{}".to_owned(),
            timestamp: now + Duration::from_millis(100),
            uuid: Uuid::new_v4()
        });
    }
    let message = rx.recv().unwrap();
}
