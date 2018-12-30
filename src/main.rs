
extern crate hyper;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

use std::sync::mpsc::Receiver;
use std::thread;
use std::env;

use hyper::mime;
use hyper::header;
use hyper::server::Server;
use hyper::client::Client;

mod scheduler;
use scheduler::Scheduler;

mod callback;
use callback::*;

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

fn main() {
    let bind = env::var("SCHEDULE_M8_BIND_ADDR")
        .unwrap_or("0.0.0.0:8001".to_owned());
    let (tx, rx) = Scheduler::spawn();
    spawn_callback_sender(rx);
    println!("Listening on {}", bind);
    Server::http(&bind)
        .unwrap()
        .handle(ServerHandler::new(tx))
        .unwrap();
}
