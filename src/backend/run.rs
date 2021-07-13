use lapin::message::Delivery;
use lapin::options::BasicAckOptions;
#[allow(unused_imports)]
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::config;
use crate::shared_models::request_response;
use crate::utils::rabbitmq::{MqChannel, MqLogin};

pub struct AppState {
  pub mq: MqChannel,
}

#[tokio::main]
pub async fn start_server() {
  dotenv::dotenv().ok();
  let matches = clap::App::new("API Web Server")
    .version("0.1.0")
    .author("Stefan Moller <stefanmoller@live.com>")
    .arg(
      clap::Arg::with_name("config")
        .short("c")
        .long("config")
        .value_name("FILE")
        .takes_value(true)
        .help("Relative location of config file"),
    )
    .get_matches();
  let _config = matches.value_of("config").unwrap_or("etc/api_worker/config.toml");

  // parse config
  let app_config = config::config_parser(_config);
  debug!("{:?}", app_config);
  let mq_config = app_config.rabbit_mq;

  let rabbit_mq_login = MqLogin {
    user: mq_config.user,
    password: mq_config.password,
    host: mq_config.host,
    port: mq_config.port,
  };

  let mq_data: HashMap<String, Vec<u8>> = HashMap::new();
  let mq_messages = Arc::new(RwLock::new(mq_data));
  let mut channel: MqChannel = MqChannel::connect(rabbit_mq_login, mq_messages);
  channel.register_exchange(&mq_config.exchange);
  channel.register_queue(&mq_config.queue);
  let routing_keys = super::get_service_keys();
  for key in routing_keys {
    channel.bind_queue(&mq_config.queue, &mq_config.exchange, &key);
  }
  let consumer = channel.consume();

  while let Some(msg) = consumer.clone().into_iter().next() {
    let state = AppState { mq: channel.clone() };
    let (_, msg) = msg.expect("error in consumer");
    let _ = msg.ack(BasicAckOptions::default()).await;
    handle_incomming_msg(msg, state).await;
  }
}

async fn handle_incomming_msg(msg: Delivery, mut state: AppState) {
  info!("incomming Message: Routing_key {}", msg.routing_key.as_str());
  let route: Vec<_> = msg.routing_key.as_str().split('.').collect();
  let payload: request_response::Request = bincode::deserialize(&msg.data).unwrap();
  let _resp: request_response::Response = match route[0] {
    "backend" => handle_backend_requests(payload),
    "" | &_ => handle_bad_requests(payload),
  };
  let resp: Vec<u8> = bincode::serialize(&_resp).unwrap();
  state.mq.reply(msg.properties, resp.to_vec()).await;
}

fn handle_bad_requests(req: request_response::Request) -> request_response::Response {
  request_response::Response {
    response_user: String::from("backend_service"),
    payload: req.payload,
    error: Some(String::from("invalid request topic")),
  }
}

fn handle_backend_requests(_req: request_response::Request) -> request_response::Response {
  request_response::Response {
    response_user: String::from("backend_service"),
    payload: "Hello from the backend".as_bytes().to_vec(),
    error: None,
  }
}
