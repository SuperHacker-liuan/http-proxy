use async_std::task;
use daemonize::Daemonize;
use std::error::Error;

mod proxy;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    daemonize();
    task::block_on(proxy::run()).expect("EXIT ");
}

fn daemonize() {
    Daemonize::new()
        .pid_file(format!("/tmp/http-proxy@{}.pid", 5555))
        .working_directory("/tmp")
        .umask(0o777)
        .start()
        .expect("Failed to start as daemon");
}
