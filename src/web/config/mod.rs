use serde_derive::Deserialize;
use std::fs;

/// Attempt to load and parse the config file into our Config struct.
/// If a file cannot be found, return a default Config.
/// If we find a file but cannot parse it, panic
pub fn config_parser(path: &str) -> Config {
  println!("Parsing config {}", path);

  let contents = fs::read_to_string(path).expect("Something went wrong reading the file");

  let config: Config = toml::from_str(&*contents).unwrap();

  config
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
  pub web_server: WebServer,
  pub redis_sessions: RedisSessions,
  pub rabbit_mq: RabbitMq,
  pub psql: Psql,
  pub prometheus: Prometheus,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebServer {
  pub host: String,
  pub port: u16,
  pub worker_pool: usize,
  pub templates: String,
  pub tls: Tls,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RedisSessions {
  pub name: String,
  pub host: String,
  pub port: u16,
  pub secure: bool,
  pub ttl: u32,
  pub private_key: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Psql {
  pub url: String,
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: String,
  pub database: String,
  pub sslmode: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Tls {
  pub cacert: String,
  pub cert: String,
  pub key: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Prometheus {
  pub endpoint: String,
  pub metrics: String,
  pub description: String,
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
  pub prefetch: u16,
}
