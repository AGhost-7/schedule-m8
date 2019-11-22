extern crate hyper;
extern crate schedule_m8;

#[test]
fn starts() {
    let mut server = schedule_m8::create_server("0.0.0.0:6161");
    server.close().unwrap();
}

#[test]
#[ignore]
fn create_callback() {
    let mut server = schedule_m8::create_server("0.0.0.0:6161");

    server.close().unwrap();
}
