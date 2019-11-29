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
    Future,
    future::{Shared, FutureExt},
    channel::oneshot,
    stream::{TryStreamExt, StreamExt}
};

pub mod callback;

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
        (&Method::POST, ["scheduler", "api"]) => {
            println!("POST -> /scheduler/api");
            //use hyper::Chunk;
            //let body = request.into_body().concat().await;
            //let callback: V1Callback = serde_json::from_str(&body)?;
            Ok(Response::new(Body::from("{}")))
        },
        (&Method::DELETE, ["scheduler", "api", id]) => {
            println!("DELETE -> /scheduler/api/{}", id);
            Ok(Response::new(Body::from("{}")))
        }
        _ =>
            Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap())
    }
}

//pub struct ScheduleM8 {
//    store_mutex: Arc<Mutex<Store>>,
//    scheduler: Scheduler,
//    server_shutdown_channel: oneshot::Sender<()>,
//    wait_handle: Box<dyn Future<Output = ()>>
//}
//
//fn print_type<T>(_: &T) {
//    println!("Type is: {}", std::any::type_name::<T>());
//}
//
//impl ScheduleM8 {
//    pub fn start(bind: String, db_path: String) -> ScheduleM8 {
//        let address: SocketAddr = bind.parse().unwrap();
//        let store_mutex = Arc::new(Mutex::new(
//                Store::open(&db_path).expect("Failed to open store"))
//        );
//        let scheduler = Scheduler::start(store_mutex.clone());
//        let (sender, receiver) = futures::channel::oneshot::channel::<()>();
//        let hyper_store = store_mutex.clone();
//        let service = make_service_fn(move |_| {
//            let service_store = store_mutex.clone();
//            async {
//                Ok::<_, GenericError>(
//                    service_fn(move |request| {
//                        handle_request(service_store.clone(), request)
//                    })
//                )
//            }
//        });
//
//        let server = Server::bind(&address).serve(service);
//
//        let wait_handle = async {
//            let graceful = server.with_graceful_shutdown(async {
//                receiver.await.ok();
//            });
//            graceful.await.unwrap();
//        };
//        print_type(&wait_handle);
//        ScheduleM8 {
//            store_mutex,
//            scheduler,
//            server_shutdown_channel: sender,
//            wait_handle: Box::new(wait_handle)
//        }
//    }
//
//    pub fn stop(self) {
//        self.scheduler.stop();
//        self.server_shutdown_channel.send(()).unwrap();
//    }
//
//    //pub async fn wait(&self) {
//    //    self.wait_handle.await;
//    //}
//    //pub fn wait(&self) -> Box<dyn Future<Output = ()>> {
//    //    self.wait_handle.clone()
//    //}
//}

pub async fn create_server(bind: String, db_path: String) {
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

    server.await.unwrap();
}
