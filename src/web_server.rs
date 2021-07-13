mod web;
use web::run::start_server;

pub mod shared_models;
pub mod utils;

#[macro_use]
extern crate json;

fn main() {
    env_logger::init();
    start_server().unwrap()
}
