
use crate::schema::Job;
use std::time::Duration;
use crate::store::Store;
use std::sync::Arc;
use chrono::Utc;
use std::str::FromStr;

use futures::channel::oneshot;

use hyper::{Client, Method, header, Request};

pub struct Scheduler {
    stop_sender: oneshot::Sender<()>
}

impl Scheduler {
    pub fn start(store: Arc<Store>) -> Scheduler {
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
                Scheduler::send_ready(&store).await;
            }
        });

        scheduler
    }

    pub fn stop(self) {
        self.stop_sender.send(()).expect("Failed to stop scheduler");
    }

    async fn send_ready(store: &Arc<Store>) {
        loop {
            let next = store.next();
            match next {
                Some(item) => {
                    if let Some(schedule) = &item.schedule {
                        let timestamp = cron::Schedule::from_str(schedule)
                            .expect("Failed to parse pattern")
                            .upcoming(Utc)
                            .next()
                            .expect("No next schedule found")
                            .timestamp_millis();
                        let callback = Job {
                            method: item.method.clone(),
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

    async fn send_callback(callback: Job) {
        let mut request = Request::new(hyper::Body::from(callback.body));
        let method = &callback.method.as_bytes();

        *request.method_mut() = Method::from_bytes(method).unwrap_or(Method::POST);
        *request.uri_mut() = callback.url.parse().expect("Invalid callback url");
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json")
        );

        let client = Client::new();

        let result = client.request(request).await;

        if let Err(e) = result {
            error!("{} - Failed to send callback: {}", callback.url, e);
        }
    }
}
