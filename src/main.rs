
use std::env;

fn main() {
    let bind = env::var("SCHEDULE_M8_BIND_ADDR")
        .unwrap_or("0.0.0.0:8001".to_owned());
    schedule_m8::create_server(&bind);
    println!("Listening on {}", bind);
}
