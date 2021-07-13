mod backend;
use backend::run::start_server;

pub mod shared_models;
pub mod utils;

fn main() {
  env_logger::init();
  start_server();
}
