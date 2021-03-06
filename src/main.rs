
#[macro_use]
extern crate log;
extern crate futures;

extern crate tokio;

use schedule_m8::ScheduleM8;
use std::env;
use env_logger::Env;

#[tokio::main]
async fn main() {
    env_logger::from_env(
        Env::default().default_filter_or("info")
    ).init();

    let mut default_dir = std::env::current_dir().unwrap();
    default_dir.push(".data");
    let bind = env::var("SCHEDULE_M8_BIND_ADDR")
        .unwrap_or("0.0.0.0:8001".to_owned());
    let db_path = env::var("SCHEDULE_M8_DATA_DIR")
        .unwrap_or(default_dir.to_str().unwrap().to_owned());

    let schedule_m8 = ScheduleM8::start(bind.clone(), db_path).await;
    info!("Listening on {}", bind);
    schedule_m8.forever().await;
}
