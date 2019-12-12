#![feature(test)]

extern crate test;
extern crate prost;
extern crate bytes;
extern crate rmp_serde;
extern crate serde;

use serde::Serialize;
use rmp_serde::Serializer;
use schedule_m8::schema::*;
use uuid::*;
use bytes::*;
use std::time::{UNIX_EPOCH, SystemTime, Duration};
use prost::Message;

#[bench]
fn prost_bench(b: &mut test::Bencher) {

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let job = ProstJob {
        method: "POST".to_owned(),
        url: "example.com".to_owned(),
        timestamp: now.as_millis() as i64,
        id: Uuid::new_v4().to_string(),
        schedule: None,
        body: "{}".to_owned()
    };

    b.iter(|| {
        let mut b = vec![];
        job.encode(&mut b).unwrap();
        let item: ProstJob = ProstJob::decode(&b).unwrap();
        //println!("testing");
    });
}

#[bench]
fn msgpack_bench(b: &mut test::Bencher) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let job = Job {
        method: "POST".to_owned(),
        url: "example.com".to_owned(),
        timestamp: now,
        id: Uuid::new_v4().to_string(),
        schedule: None,
        body: "{}".to_owned()
    };
    b.iter(move || {
        let mut buffer = Vec::new();
        let mut serializer = Serializer::new(&mut buffer);
        job.serialize(&mut serializer).unwrap();
        let item: Job = rmp_serde::decode::from_slice(&buffer).unwrap();
    });
}
