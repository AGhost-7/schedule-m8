
use std::env;

fn main() {
    let bind = env::var("SCHEDULE_M8_BIND_ADDR")
        .unwrap_or("0.0.0.0:8001".to_owned());
    let db_path = env::var("SCHEDULE_M8_DATA_DIR")
        .unwrap_or("/var/lib/schedule-m8/data".to_owned());
    println!("Listening on {}", bind);
    schedule_m8::create_server(&bind, &db_path);
}
