
use crate::callback::Callback;
use std::time::Duration;
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
        self.stop_sender.send(()).expect("Failed to stop scheduler");
    }

    fn pop_next(store_mutex: &Arc<Mutex<Store>>) -> Option<Callback> {
        store_mutex
            .lock()
            .expect("Failed to acquire store lock due to poisoning")
            .next()
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
                            .expect("Invalid schedule")
                            .upcoming(Utc)
                            .next()
                            .expect("Cannot find the next time for schedule")
                            .timestamp();
                        let callback = Callback {
                            timestamp: Duration::from_millis(timestamp as u64),
                            url: item.url.clone(),
                            body: item.body.clone(),
                            id: item.id.clone(),
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
        let mut request = Request::new(hyper::Body::from(callback.body));

        *request.method_mut() = Method::POST;
        *request.uri_mut() = callback.url.parse().expect("Invalid callback url");
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
