extern crate tokio;
extern crate schedule_m8;
extern crate hyper;
extern crate serde_json;
extern crate rand;
extern crate futures;

use schedule_m8::schema::*;
use schedule_m8::ScheduleM8;

use hyper::{Client, Server, Body, Request, Response, Method};
use hyper::service::{make_service_fn, service_fn};

use std::time::{UNIX_EPOCH, SystemTime, Duration};
use std::sync::{Mutex, Arc};

use bytes::buf::BufExt;
use uuid::Uuid;

fn random_port() -> u16 {
    use rand::prelude::*;
    let mut rng = rand::thread_rng();
    rng.gen_range(7001, 10000)
}

macro_rules! test_case {
    ($name:ident $test:expr) => {
        #[tokio::test]
        async fn $name() {
            let app_port = random_port();
            let data_dir = ".test/".to_owned() + &Uuid::new_v4().to_string();
            let app = ScheduleM8::start(
                "0.0.0.0:".to_owned() + &app_port.to_string(),
                data_dir.clone()
            ).await;
            
            let client = Client::new();

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
            tokio::spawn(async move {
                server.await.unwrap();
            });

            // run actual test...
            $test

            // end of actual test
            app.stop();
            tokio::fs::remove_dir_all(&data_dir).await.expect("Error removing directory");
        }
    }
}

test_case!(create_callback {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let callback = V1Job {
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
    let mut interval = tokio::time::interval(Duration::from_millis(2000));
    interval.tick().await;
    interval.tick().await;
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
});

test_case!(cancel_callback {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    let callback = V1Job {
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
    let response = client.request(request).await.unwrap();

    let body = hyper::body::aggregate(response).await.unwrap();
    let key: V1JobKey = serde_json::from_reader(body.reader()).unwrap();

    request = Request::new(
        Body::from("")
    );

    *request.uri_mut() = (
        "http://localhost:".to_owned() +
            &app_port.to_string() +
            "/scheduler/api/" +
            (&key.key.to_string())
    ).parse().unwrap();
    *request.method_mut() = Method::DELETE;
    client.request(request).await.unwrap();

    let mut interval = tokio::time::interval(Duration::from_millis(2000));
    interval.tick().await;
    interval.tick().await;

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 0);
});

test_case!(delete_triggered_callback {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    let callback = V1Job {
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
    let response = client.request(request).await.unwrap();

    let mut interval = tokio::time::interval(Duration::from_millis(2000));
    interval.tick().await;
    interval.tick().await;


    let body = hyper::body::aggregate(response).await.unwrap();
    let key: V1JobKey = serde_json::from_reader(body.reader()).unwrap();

    request = Request::new(
        Body::from("")
    );
    *request.uri_mut() = (
        "http://localhost:".to_owned() +
        &app_port.to_string() +
        "/scheduler/api/" +
        (&key.key.to_string())
    ).parse().unwrap();
    *request.method_mut() = Method::DELETE;
    let response = client.request(request).await.unwrap();
    assert_eq!(response.status(), 404);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
});

test_case!(missing_id {
    let mut request = Request::new(
        Body::from("")
    );
    *request.uri_mut() = (
        "http://localhost:".to_owned() +
        &app_port.to_string() +
        "/scheduler/api/boom"
    ).parse().unwrap();
    *request.method_mut() = Method::DELETE;
    let response = client.request(request).await.unwrap();
    assert_eq!(response.status(), 404);
});
