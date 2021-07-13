use actix_redis::{RedisActor, RedisSession};
use actix_web::http::header;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_prom::PrometheusMetrics;
use futures::executor::block_on;
#[allow(unused_imports)]
use log::{debug, error, info};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use prometheus::{opts, IntCounterVec};
use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, RwLock},
  thread,
};
use tera::Tera;

use super::config;
use super::router;
use super::AppState;
use crate::utils::rabbitmq::{MqChannel, MqLogin};

const WEBSERVICE_KEY: &str = "reply.web_service";

const SERVICEKEYS: [&str; 1] = [WEBSERVICE_KEY];

pub fn get_service_keys() -> HashSet<String> {
  let mut service_keys: HashSet<String> = HashSet::new();
  for key in SERVICEKEYS.iter() {
    service_keys.insert(key.to_string());
  }
  service_keys
}

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
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
  let _config = matches
    .value_of("config")
    .unwrap_or("etc/api_server/config.toml");

  // Parse config
  let app_config = config::config_parser(_config);
  // Set local config strings
  info!("{:?}", app_config.clone());
  let _session_conf = app_config.redis_sessions;
  let _mq_config = app_config.rabbit_mq;
  let _server_config = app_config.web_server.clone();

  // Configure RabbitMq
  let rabbit_mq_login = MqLogin {
    user: _mq_config.user,
    password: _mq_config.password,
    host: _mq_config.host,
    port: _mq_config.port,
  };
  let mq_data: HashMap<String, Vec<u8>> = HashMap::new();
  let mq_messages = Arc::new(RwLock::new(mq_data));
  let mut channel: MqChannel = MqChannel::connect(rabbit_mq_login, mq_messages);
  channel.register_exchange(&_mq_config.exchange);
  channel.set_qos(_mq_config.prefetch);
  channel.register_queue(&_mq_config.queue);
  let routing_keys = get_service_keys();
  channel.bind_queue(&_mq_config.queue, &_mq_config.exchange, &_mq_config.queue);
  for key in routing_keys {
    channel.bind_queue(&_mq_config.queue, &_mq_config.exchange, &key);
  }
  let mut consuming_channel = channel.clone();
  thread::spawn(move || {
    block_on(consuming_channel.start_consuming());
  });
  info!("Rabbitmq loaded and ready");

  // configure tls
  let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
  builder
    .set_private_key_file(app_config.web_server.tls.key, SslFiletype::PEM)
    .unwrap();
  builder
    .set_certificate_chain_file(app_config.web_server.tls.cert)
    .unwrap();

  // prometheus config
  let prometheus = PrometheusMetrics::new("api", Some(&app_config.prometheus.endpoint), None);
  let counter_opts = opts!(
    &app_config.prometheus.metrics,
    &app_config.prometheus.description
  )
  .namespace("api");
  let counter = IntCounterVec::new(counter_opts, &["endpoint", "method", "status"]).unwrap();
  // Run http server
  HttpServer::new(move || {
    let tera = Tera::new(&_server_config.templates).unwrap();
    // Redis Cache configuration
    let redis_addr = RedisActor::start("127.0.0.1:6379");
    // Configure App State
    let state = AppState {
      mq: channel.clone(),
      tmpl: tera,
      metrics: counter.clone(),
    };
    // Configure Session
    let session = RedisSession::new(
      format!("{}:{}", _session_conf.host, _session_conf.port),
      _session_conf.private_key.as_bytes(),
    )
    .cookie_name(&_session_conf.name)
    .ttl(_session_conf.ttl)
    .cookie_secure(_session_conf.secure);
    App::new()
      .wrap(Logger::default())
      .wrap(Logger::new("%a %{User-Agent}i"))
      .wrap(Compress::default())
      .wrap(session)
      .wrap(prometheus.clone())
      .data(state)
      .data(redis_addr)
      .configure(router::routes::dispatcher)
      .default_service(web::to(default_service))
  })
  .workers(app_config.web_server.worker_pool)
  .bind(format!(
    "{}:{}",
    app_config.web_server.host, app_config.web_server.port
  ))?
  // .bind_openssl(
  //   format!(
  //     "{}:{}",
  //     app_config.web_server.host, app_config.web_server.port
  //   ),
  //   builder,
  // )?
  .run()
  .await
}

async fn default_service(state: web::Data<AppState>, _req: HttpRequest) -> HttpResponse {
  let x: &str = _req
    .headers()
    .get(header::USER_AGENT)
    .unwrap()
    .to_str()
    .unwrap();
  match x.contains("curl") {
    true => {
      let err_string = object! {
          error: 404,
          message: "unknown path",
          url: _req.path(),
      };
      HttpResponse::Ok().json(err_string.dump())
    }
    false => {
      let ctx = tera::Context::new();
      match state.tmpl.render("404.html", &ctx) {
        Ok(s) => return HttpResponse::Ok().content_type("text/html").body(s),
        Err(e) => {
          error!("failed to load template: {}", e);
          let err_string = object! {
              error: "404",
              message: "unknown path",
              url: _req.path(),
          };
          HttpResponse::Ok().json(err_string.dump())
        }
      }
    }
  }
}
