extern crate tokio;
extern crate tokio_timer;
extern crate schedule_m8;
extern crate hyper;
extern crate serde_json;
extern crate rand;
extern crate futures;

use schedule_m8::callback::V1Callback;
use schedule_m8::ScheduleM8;

use hyper::{Client, Server, Body, Request, Response, Method};
use hyper::service::{make_service_fn, service_fn};

use std::time::{UNIX_EPOCH, SystemTime, Duration, Instant};
use std::sync::{Mutex, Arc};

#[tokio::main]
#[test]
async fn starts() {
    let server = ScheduleM8::start("0.0.0.0:6161".to_owned(), ".test/starts".to_owned());
    server.stop();
}


fn random_port() -> u16 {
    use rand::prelude::*;
    let mut rng = rand::thread_rng();
    rng.gen_range(7001, 10000)
}

#[tokio::main]
#[test]
async fn create_callback() {
    let app_port = random_port();
    let app = ScheduleM8::start(
        "0.0.0.0:".to_owned() + &app_port.to_string(),
        ".test/create_callback".to_owned()
    );
    
    let client = Client::new();

    {
        let mut request = Request::new(Body::from(""));
        *request.method_mut() = Method::DELETE;
        let uri = "http://127.0.0.1:".to_owned() + &app_port.to_string() + "/scheduler/api";
        *request.uri_mut() = uri.parse().unwrap();
        client.request(request).await.expect("Failed to clear queue");
    }

    let requests: Arc<Mutex<Vec<Request<Body>>>> = Arc::new(Mutex::new(Vec::new()));
    let service_requests = requests.clone();
    let service = make_service_fn(move|_| {
        let requests = service_requests.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                requests
                    .lock()
                    .expect("Failed to lock requests vec")
                    .push(req);
                async {
                    Ok::<Response<Body>, hyper::Error>(
                        Response::new(Body::from("{}"))
                    )
                }
            }))
        }
    });

    let server_port = random_port();
    let server_address = ([0, 0, 0, 0], server_port).into();
    let server = Server::bind(&server_address).serve(service);
    hyper::rt::spawn(async move {
        server.await.unwrap();
    });

    // actual test...
    {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let callback = V1Callback {
            payload: "{}".to_owned(),
            timestamp: (now + 1000) as u64,
            url: "http://127.0.0.1:".to_owned() + &server_port.to_string() + "/test",
        };

        let mut request = Request::new(
            Body::from(serde_json::to_string(&callback).unwrap())
        );
        *request.uri_mut() = (
            "http://localhost:".to_owned() + &app_port.to_string() + "/scheduler/api"
        ).parse().unwrap();
        *request.method_mut() = Method::POST;
        client.request(request).await.unwrap();
        let mut interval = tokio_timer::Interval::new_interval(Duration::from_millis(2000));
        interval.next().await;
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
    }
    // end of actual test
    app.stop();
}

#[tokio::main]
#[test]
#[ignore]
async fn cancel_callback() {
}

#[tokio::main]
#[test]
#[ignore]
async fn delete_triggered_callback() {
}
