
use crate::callback::Callback;
use std::time::{UNIX_EPOCH, Duration, SystemTime};
use crate::store::Store;
use std::sync::{Mutex,Arc};
use std::str::FromStr;
use cron::Schedule;
use chrono::Utc;

use futures::channel::oneshot;

use hyper::{Client, Method, header, Request};

pub struct Scheduler {
    stop_sender: oneshot::Sender<()>
}

impl Scheduler {
    pub fn start(store_mutex: Arc<Mutex<Store>>) -> Scheduler {
        let (sender, mut receiver) = futures::channel::oneshot::channel::<()>();
        let mut interval = tokio_timer::Interval::new_interval(Duration::from_millis(500));
        let scheduler = Scheduler {
            stop_sender: sender
        };
        hyper::rt::spawn(async move {
            loop {
                interval.next().await;
                match receiver.try_recv() {
                    Err(_) => panic!("Scheduler shutdown handler cancelled"),
                    Ok(Some(())) => break,
                    Ok(None) => {}
                }
                Scheduler::send_ready(&store_mutex).await;
            }
        });

        scheduler
    }

    pub fn stop(self) {
        self.stop_sender.send(()).unwrap();
    }

    fn pop_next(store_mutex: &Arc<Mutex<Store>>) -> Option<Callback> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error getting system time");
        let mut store = store_mutex
            .lock()
            .expect("Failed to acquire store lock due to poisoning");
        if let Some(item) = store.peek() {
            if item.timestamp.lt(&now) || item.timestamp.eq(&now) {
                return store.pop()
            }
        }
        return None
    }

    async fn send_ready(store_mutex: &Arc<Mutex<Store>>) {
        loop {
            let next = Scheduler::pop_next(store_mutex);
            match next {
                Some(item) => {
                    if let Some(schedule) = &item.schedule {
                        let mut store = store_mutex
                            .lock()
                            .expect("Failed to acquire store lock due to poisoning");
                        let timestamp = Schedule::from_str(schedule)
                            .unwrap()
                            .upcoming(Utc)
                            .next()
                            .unwrap()
                            .timestamp();
                        let callback = Callback {
                            timestamp: Duration::from_millis(timestamp as u64),
                            url: item.url.clone(),
                            body: item.body.clone(),
                            uuid: item.uuid.clone(),
                            schedule: Some(schedule.to_owned())
                        };
                        store.push(callback);
                    }
                    Scheduler::send_callback(item).await;
                },
                None => break
            }
        }
    }

    async fn send_callback(callback: Callback) {
        let uri: hyper::Uri = callback.url.parse().expect("Invalid callback url");
        let mut request = Request::new(hyper::Body::from(callback.body));

        *request.method_mut() = Method::POST;
        *request.uri_mut() = uri;
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json")
        );

        let client = Client::new();

        let result = client.request(request).await;

        if let Err(e) = result {
            eprintln!("{} - Failed to send callback: {}", callback.url, e);
        }
    }
}
