#[macro_use]
extern crate log;
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
extern crate cron_parser;

use std::net::SocketAddr;
use std::sync::Arc;

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

use std::convert::TryFrom;

mod error;

pub mod schema;
use crate::schema::*;

mod scheduler;
use scheduler::Scheduler;

mod store;
use crate::store::Store;

type GenericError = Box<dyn std::error::Error + Send + Sync>;

async fn request_routes(
    store: Arc<Store>,
    request: Request<Body>
) -> Result<Response<Body>, GenericError> {
    let parts: Vec<&str> = request
        .uri()
        .path()
        .split("/")
        .filter(|part| !part.is_empty())
        .collect();
    match (request.method(), parts.as_slice()) {
        // {{{ v1
        (&Method::POST, ["scheduler", "api", "cron"]) => {
            info!("POST -> /scheduler/api/cron");
            let body = request.into_body().try_concat().await?;
            let str_body = String::from_utf8(body.to_vec())?;
            let v1_job: V1CronJob = serde_json::from_str(&str_body)?;
            let job = Job::try_from(v1_job)?;
            store.push(job);
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::DELETE, ["scheduler", "api"]) => {
            info!("DELETE -> /scheduler/api");
            store.clear();
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::POST, ["scheduler", "api"]) => {
            info!("POST -> /scheduler/api");
            let body = request.into_body().try_concat().await?;
            let str_body = String::from_utf8(body.to_vec())?;
            let v1_job: V1Job = serde_json::from_str(&str_body)?;
            let job = Job::from(v1_job);
            let key = V1JobKey::new(job.id.clone());

            store.push(job);
            Ok(Response::new(Body::from(serde_json::to_string(&key)?)))
        },
        (&Method::DELETE, ["scheduler", "api", id]) => {
            info!("DELETE -> /scheduler/api/{}", id);
            let removed = store.remove(&id);
            match removed {
                Some(_) => Ok(Response::new(Body::from("{}"))),
                None =>
                    Ok(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header("Content-Type", "application/json")
                            .body(Body::from("{}"))
                            .unwrap()
                    )
            }
        },
        // }}}
        // {{{ v2
        (&Method::POST, ["api", "job"]) => {
            info!("POST -> /api/job");
            let body = request.into_body().try_concat().await?;
            let str_body = String::from_utf8(body.to_vec())?;
            let v2_job: V1Job = serde_json::from_str(&str_body)?;
            let job = Job::from(v2_job);
            let response = serde_json::to_string(&V2JobResponse::from(&job))?;
            store.push(job);
            Ok(Response::new(Body::from(response)))
        },
        (&Method::DELETE, ["api", "job"]) => {
            info!("DELETE -> /scheduler/api");
            store.clear();
            Ok(
                Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::from(""))
                    .unwrap()
            )
        },
        (&Method::DELETE, ["api", "job", id]) => {
            info!("DELETE -> /api/job/{}", id);
            let removed = store.remove(&id);
            match removed {
                Some(_) => Ok(
                    Response::builder()
                        .status(StatusCode::NO_CONTENT)
                        .body(Body::from(""))
                        .unwrap()
                ),
                None =>
                    Ok(
                        Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from(""))
                            .unwrap()
                    )
            }
        },
        (&Method::POST, ["api", "cron"]) => {
            info!("POST -> /api/cron");
            let body = request.into_body().try_concat().await?;
            let str_body = String::from_utf8(body.to_vec())?;
            let v2_job: V2CronJob = serde_json::from_str(&str_body)?;
            let job = Job::try_from(v2_job)?;
            let response = serde_json::to_string(&V2CronJobResponse::from(&job))?;
            store.push(job);
            Ok(Response::new(Body::from(response)))
        },
        // }}}
        (method, parts) => {
            info!("{} -> {}: NOT_FOUND", method, parts.join("/"));
            Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap()
            )
        }
    }
}

async fn handle_request(
        store: Arc<Store>,
        request: Request<Body>
        ) -> Result<Response<Body>, GenericError> {
    let result = request_routes(store, request).await;
    result.or_else(|err| {
        error!("Error: {}", err);
        Ok(
            Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .body(Body::from("{}"))
            .unwrap()
        )
    })
}

pub struct ScheduleM8 {
    scheduler: Scheduler,
    close_sender: oneshot::Sender<()>,
    closed_receiver: oneshot::Receiver<()>
}

impl ScheduleM8 {
    pub fn start(bind: String, db_path: String) -> ScheduleM8 {
        info!("Opening store at location: {}", db_path);
        let store = Arc::new(Store::open(&db_path).expect("Failed to open store"));
        let scheduler = Scheduler::start(store.clone());

        let address: SocketAddr = bind.parse().unwrap();

        let make_svc = make_service_fn(move |_| {
            let service_store = store.clone();
            async {
                Ok::<_, GenericError>(service_fn(move |req| {
                    handle_request(service_store.clone(), req)
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
