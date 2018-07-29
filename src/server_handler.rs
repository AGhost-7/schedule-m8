extern crate hyper;
extern crate serde;
extern crate serde_json;

use callback::*;

use hyper::server::{Handler, Request, Response};
use std::sync::{Mutex};
use std::sync::mpsc::{Sender};

pub struct ServerHandler {
    tx: Mutex<Sender<Callback>>
}

macro_rules! send_json {
    ($res:ident, $js:tt) => {
        {
            let response_body = json!($js);
            let serialized = serde_json::to_vec(&response_body).expect("Serialization error.");
            $res.send(&serialized).expect("IO Error sending response body.");
        }
    }
}

impl ServerHandler {

    pub fn new(tx: Sender<Callback>) -> ServerHandler {
        ServerHandler {
            tx: Mutex::new(tx)
        }
    }

    fn v1_parse(&self, req: &mut Request) -> Result<V1Callback, serde_json::Error> {
        serde_json::from_reader(req)
    }

    fn v1_post(&self, req: &mut Request, mut res: Response) {
        match self.v1_parse(req) {
            Ok(parsed) => {
                let schedulable = parsed.to_schedulable();
                let key = "http::".to_owned() + &schedulable.uuid.simple().to_string();
                self
                    .tx
                    .lock()
                    .expect("Channel mutex has been poisoned")
                    .send(schedulable)
                    .expect("Failed to send message to scheduler - channel disconnected");
                *res.status_mut() = hyper::Ok;
                send_json!(res, {
                    "key": key
                });
            },
            Err(e) => {
                println!("Failed to parse json: {}", e);
                *res.status_mut() = hyper::BadRequest;
                send_json!(res, {
                    "message": "Failed to parse json"
                });
            }
        }
    }
}

impl Handler for ServerHandler {
    fn handle(&self, mut req: Request, mut res: Response) {
        println!("{} => {}", req.method, req.uri);
        match req.uri.clone() {
            hyper::uri::RequestUri::AbsolutePath(url) => {
                match (&req.method, &*url) {
                    (&hyper::Post, "/scheduler/api") => {
                        self.v1_post(&mut req, res);
                    },
                    (&hyper::Post, "/api/v1/schedule") => {
                        self.v1_post(&mut req, res);
                    },
                    _ => {
                        *res.status_mut() = hyper::NotFound;
                    }
                }
            },
            _ => {
                println!("Not abs path");
                *res.status_mut() = hyper::BadRequest;
            }
        }

    }
}
