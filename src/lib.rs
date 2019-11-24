#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate uuid;

extern crate hyper;

extern crate sled;
extern crate priority_queue;
extern crate rmp_serde;

use std::sync::mpsc::Receiver;
use std::thread;

use hyper::server::Server;
use hyper::client::Client;
use hyper::mime;
use hyper::header;
use std::path::Path;

mod callback;
use callback::*;

mod scheduler;
use scheduler::Scheduler;

mod store;
use store::Store;

mod server_handler;
use server_handler::ServerHandler;

fn spawn_callback_sender(rx: Receiver<Callback>) {
    thread::spawn(move || {
        let client = Client::new();
        loop {
            let callback = rx.recv().unwrap();

            let content_type = header::ContentType(
                mime::Mime(
                    mime::TopLevel::Application, mime::SubLevel::Json, Vec::new()
                )
            );
            client
                .post(&callback.url)
                .header(content_type)
                .body(&callback.body)
                .send()
                .unwrap();
        }
    });
}

pub fn create_server(bind: &str, db_path: &str) -> hyper::server::Listening {
    let store = Store::open(db_path).unwrap();
    let (tx, rx) = Scheduler::spawn(store);
    spawn_callback_sender(rx);
    Server::http(bind)
        .unwrap()
        .handle(ServerHandler::new(tx))
        .unwrap()
}
