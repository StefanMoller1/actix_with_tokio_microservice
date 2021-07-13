use serde::Deserialize;
use std::fs;

pub fn config_parser(path: &str) -> Config {
  println!("Parsing config {}", path);

  let contents = fs::read_to_string(path).expect("Something went wrong reading the file");

  let config: Config = toml::from_str(&*contents).unwrap();

  config
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
  // pub web_server: WebServer,
  // pub cookie: Cookie,
  // pub mesh: Mesh,
  // pub apid: Apid,
  pub rabbit_mq: RabbitMq,
  pub redis: Redis,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMq {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: String,
  pub exchange: String,
  pub queue: String,
  pub consumer: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Redis {
  pub host: String,
  pub port: u16,
  pub private_key: String,
}
