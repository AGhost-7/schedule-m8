
// Callback queue has constant time lookup to see if there
// is a callback to send or not. This is important as we
// will be checking very often (every couple millisecs).
//
// O(log n) for insertions (worst case) due to sorting.

use std::vec::Vec;
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use callback::Callback;
use std::time::{UNIX_EPOCH, Duration, SystemTime};

pub struct Scheduler {
    rx: Receiver<Callback>,
    tx: Sender<Callback>,
    elems: Vec<Callback>
}

impl Scheduler {

    pub fn new(tx: Sender<Callback>, rx: Receiver<Callback>) -> Scheduler {
        Scheduler {
            rx: rx,
            tx: tx,
            elems: Vec::new()
        }
    }

    pub fn spawn() -> (Sender<Callback>, Receiver<Callback>) {
        let (tx, rx) = channel::<Callback>();
        let (trigger_tx, trigger_rx) = channel::<Callback>();
        let mut handler = Scheduler::new(trigger_tx, rx);
        thread::spawn(move|| {
            loop {
                handler.check_received();
                handler.check_to_send();
                // Since reads are so efficient this seems to be
                // an ok sleep time.
                thread::sleep(Duration::from_millis(100));
            }
        });
        (tx, trigger_rx)
    }

    pub fn check_received(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(callback) => {
                    println!("Added callback");
                    self.elems.push(callback);
                    self.elems.sort();
                },
                Err(TryRecvError::Empty) => {
                    return;
                },
                Err(_) => {
                    println!("Thread event bus disconnected");
                    panic!();
                }
            }
        }
    }

    fn should_pop(&self, now: &Duration) -> bool {
        match self.elems.last() {
            Some(elem) => elem.timestamp.lt(now) || elem.timestamp.eq(now),
            None => false
        }
    }

    pub fn check_to_send(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Error getting system time");
        while self.should_pop(&now) {
            let elem = self.elems.pop().unwrap();
            self.tx.send(elem).expect("Scheduler channel disconnected.");
        }
    }
}

#[test]
fn scheduling_callback() {
    use uuid::Uuid;

    let (tx, rx) = Scheduler::spawn();
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
        println!("Got url {}", res.url);
        assert!(&res.url == i);
    }
}
