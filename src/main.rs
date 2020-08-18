use self::config::CONFIG;
use async_std::task;
use daemonize::Daemonize;
use std::error::Error;

mod config;
mod proxy;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    log::info!("Proxy listen on {}", CONFIG.listen);
    daemonize();
    task::block_on(proxy::run()).expect("EXIT ");
}

fn daemonize() {
    if !CONFIG.daemon {
        return;
    }
    Daemonize::new()
        .pid_file(format!("/tmp/http-proxy@{}.pid", CONFIG.listen))
        .working_directory("/tmp")
        .umask(0o777)
        .start()
        .expect("Failed to start as daemon");
}
