use crate::utils::rabbitmq::MqChannel;
use prometheus::IntCounterVec;
use tera::Tera;

mod config;
pub mod router;
pub mod run;

pub struct AppState {
  pub mq: MqChannel,
  pub tmpl: Tera,
  pub metrics: IntCounterVec,
}
