#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod callback;
use callback::{Callback};

mod scheduler;
use scheduler::{Scheduler};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender};

use std::env;

extern crate hyper;
use hyper::mime;
use hyper::header;
use hyper::server::{Handler, Server, Request, Response};
use hyper::client::Client;

#[derive(Deserialize, Serialize)]
struct V1Callback {
    timestamp: u32,
    url: String,
    payload: String
}

struct ServerHandler {
    tx: Mutex<Sender<Callback>>,
    client: Arc<Client>
}

impl ServerHandler {

    fn v1_parse(&self, req: &mut Request) -> Result<V1Callback, serde_json::Error> {
        serde_json::from_reader(req)
    }

    fn v1_post(&self, req: &mut Request, mut res: Response) {
        match self.v1_parse(req) {
            Ok(parsed) => {
                let content_type = header::ContentType(
                    mime::Mime(
                        mime::TopLevel::Application, mime::SubLevel::Json, Vec::new()
                    )
                );
                self
                    .client
                    .post(&parsed.url)
                    .header(content_type)
                    .body(&parsed.payload)
                    .send()
                    .unwrap();
                *res.status_mut() = hyper::Ok;
                res.send(b"{\"key\":\"foo::bar\"}").unwrap();
            },
            Err(_) => {
                *res.status_mut() = hyper::BadRequest;
                res.send(b"Error").unwrap();
            }
        }
    }
}

impl Handler for ServerHandler {
    fn handle(&self, mut req: Request, mut res: Response) {
        match req.uri.clone() {
            hyper::uri::RequestUri::AbsolutePath(url) => {
                match (&req.method, &*url) {
                    (&hyper::Post, "/api/v1/schedule") => {
                        self.v1_post(&mut req, res);
                    },
                    _ => {
                        *res.status_mut() = hyper::NotFound;
                    }
                }
            },
            _ => {
                *res.status_mut() = hyper::BadRequest;
            }
        }

    }
}

fn main() {
    let bind = env::var("SCHEDULE_M8_BIND_ADDR")
        .unwrap_or("0.0.0.0:8001".to_owned());
    let (tx, _) = Scheduler::spawn();
    let client = Client::new();
    Server::http(&bind)
        .unwrap()
        .handle(ServerHandler {
            tx: Mutex::new(tx),
            client: Arc::new(client)
        })
        .unwrap();
}
