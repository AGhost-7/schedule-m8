extern crate schedule_m8;
extern crate hyper;
extern crate serde_json;

extern crate futures;

use schedule_m8::callback::V1Callback;
use hyper::rt::Future;

use hyper::{
    Client,
    http::Request
};

#[test]
#[ignore]
fn starts() {
    let mut close = schedule_m8::create_server("0.0.0.0:6161", ".test/data");
    //close.send(());
}

#[test]
#[ignore]
fn create_callback() {
    schedule_m8::create_server("0.0.0.0:6161", ".test/data");
    let client = Client::new();

    let callback = V1Callback {
        timestamp: 1,
        url: "http://localhost:6161/foo".to_owned(),
        payload: "{}".to_owned()
    };

    let body: Vec<u8> = serde_json::to_vec(&callback).unwrap();
    use hyper::body::Body;
    let request: Request<Body> = Request::post("http://localhost:6161/scheduler/api")
        .body(Body::from(body))
        .expect("Failed to build request");

    //let future = client.request(request).map(|response| {
    //    println!("Got response");
    //    Ok(())
    //});

    //hyper::rt::run(future);
    //future.wait();

    //close.send(());
}
