use async_std::task;
use std::error::Error;

mod proxy;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    task::block_on(proxy::run()).expect("EXIT ");
}
