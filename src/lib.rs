extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

extern crate sled;
extern crate priority_queue;
extern crate rmp_serde;

#[macro_use]
extern crate warp;
extern crate hyper;
extern crate futures;

use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use warp::Filter;
use hyper::{
    Client,
    body::Body
};
use futures::Future;

pub mod callback;
use callback::*;

mod scheduler;
use scheduler::Scheduler;

mod store;
use store::Store;

fn spawn_callback_sender(rx: Receiver<Callback>) {
    thread::spawn(move || {
        let client = Client::new();
        loop {
            let callback = rx.recv().unwrap();
            let request = hyper::Request::builder()
                .method("POST")
                .uri(callback.url)
                .header("Content-Type", "application/json")
                .body(Body::from(callback.body))
                .expect("Failed to build request");
            let future = client
                .request(request)
                .map_err(|| ())
                .map(|| ());
            hyper::rt::run(future);
            //future.wait();
        }
    });
}

pub fn create_server(bind: &str, db_path: &str) { //-> futures::Complete<()> {
    let store = Arc::new(Mutex::new(Store::open(db_path).unwrap()));
    let (_, rx) = Scheduler::spawn(store.clone());
    spawn_callback_sender(rx);

    let warp_store = store.clone();
    let store_filter = warp::any().map(move || warp_store.clone());
    let create_callback = warp::post2()
        .and(path!("scheduler" / "api"))
        .and(warp::body::json())
        .and(store_filter)
        .map(|callback: V1Callback, store: Arc<Mutex<Store>>| {
            println!("Got callback {:?}", callback);
            let mut store = store.lock().expect("Failed to acquire lock on storage");
            store.push(callback.to_schedulable());
            "".to_owned()
        });

    let api = create_callback;
    //let (bind_tx, bind_rx) = futures::channel::oneshot::channel();
    let address: SocketAddr = bind.parse().unwrap();
    warp::serve(api).run(address);//bind_with_graceful_shutdown(address, bind_rx);

    //bind_tx
}
