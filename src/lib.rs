extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

extern crate sled;
extern crate priority_queue;
extern crate rmp_serde;

extern crate hyper;
extern crate tokio_timer;
extern crate futures;
extern crate cron;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use hyper::{
    body::Body,
    Response,
    Server,
    Request,
    StatusCode,
    Method,
    service::{make_service_fn, service_fn}
};

use futures::{
    stream::TryStreamExt,
    channel::oneshot
};

use uuid::Uuid;

pub mod callback;
use crate::callback::*;

mod scheduler;
use scheduler::Scheduler;

mod store;
use crate::store::Store;

type GenericError = Box<dyn std::error::Error + Send + Sync>;

async fn handle_request(
        store_mutex: Arc<Mutex<Store>>,
        request: Request<Body>
        ) -> Result<Response<Body>, GenericError> {
    let parts: Vec<&str> = request
        .uri()
        .path()
        .split("/")
        .filter(|part| !part.is_empty())
        .collect();
    match (request.method(), parts.as_slice()) {
        (&Method::POST, ["scheduler", "api", "cron"]) => {
            println!("POST -> /scheduler/api/cron");
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::DELETE, ["scheduler", "api", "cron", id]) => {
            println!("DELETE -> /scheduler/api/cron/{}", id);
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::DELETE, ["scheduler", "api"]) => {
            println!("DELETE -> /scheduler/api");
            store_mutex
                    .lock()
                    .expect("Failed to acquire lock on storage")
                    .clear();
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::POST, ["scheduler", "api"]) => {
            println!("POST -> /scheduler/api");
            let body = request.into_body().try_concat().await?;
            let str_body = String::from_utf8(body.to_vec())?;
            let v1_callback: V1Callback = serde_json::from_str(&str_body)?;
            let callback = Callback::from(v1_callback);
            let key = V1CallbackKey::new(callback.uuid);

            store_mutex
                .lock()
                .expect("Failed to acquire lock")
                .push(callback);
            Ok(Response::new(Body::from(serde_json::to_string(&key)?)))
        },
        (&Method::DELETE, ["scheduler", "api", id]) => {
            println!("DELETE -> /scheduler/api/{}", id);
            store_mutex
                    .lock()
                    .expect("Failed to acquire lock on storage")
                    .remove(&Uuid::parse_str(id)?);
            Ok(Response::new(Body::from("{}")))
        },
        (method, parts) => {
            println!("{} -> {}: NOT_FOUND", method, parts.join("/"));
            Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap())
        }
    }
}

pub struct ScheduleM8 {
    scheduler: Scheduler,
    close_sender: oneshot::Sender<()>,
    closed_receiver: oneshot::Receiver<()>
}

impl ScheduleM8 {
    pub fn start(bind: String, db_path: String) -> ScheduleM8 {
        println!("Opening store at location: {}", db_path);
        let store_mutex = Arc::new(Mutex::new(Store::open(&db_path).expect("Failed to open store")));
        let scheduler = Scheduler::start(store_mutex.clone());

        let address: SocketAddr = bind.parse().unwrap();

        let make_svc = make_service_fn(move |_| {
            let service_store_mutex = store_mutex.clone();
            async {
                Ok::<_, GenericError>(service_fn(move |req| {
                    handle_request(service_store_mutex.clone(), req)
                }))
            }
        });

        let server: Server<_,_> = Server::bind(&address).serve(make_svc);
        let (close_sender, close_receiver) = oneshot::channel::<()>();
        let (closed_sender, closed_receiver) = oneshot::channel::<()>();

        hyper::rt::spawn(async {
            server
                .with_graceful_shutdown(async {
                    close_receiver.await.ok();
                })
            .await
                .unwrap();
            closed_sender.send(()).unwrap();
        });

        ScheduleM8 {
            scheduler,
            close_sender,
            closed_receiver
        }
    }

    pub fn stop(self) {
        self.scheduler.stop();
        self.close_sender.send(()).unwrap();
    }

    pub async fn forever(self) {
        self.closed_receiver.await.unwrap();
    }
}
