use lapin::{
  options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
  },
  types::FieldTable,
  types::ShortString,
  BasicProperties, Channel, Connection, ConnectionProperties, Consumer, ExchangeKind,
};
use log::{debug, error, info};
use std::{
  collections::HashMap,
  str,
  sync::{Arc, RwLock},
  time::SystemTime,
};
use uuid::Uuid;

#[allow(dead_code)]
const MSG_EXPIRATION: &str = "20000";
#[allow(dead_code)]
const ERROR: &str = "[ERROR]";
#[allow(dead_code)]
const WARNING: &str = "[WARNING]";
#[allow(dead_code)]
const TIMEOUT: &str = "[TIMEOUT]";

pub struct MqLogin {
  pub user: String,
  pub password: String,
  pub host: String,
  pub port: String,
}

#[derive(Debug, Clone)]
pub struct MqChannel {
  pub channel: Channel,
  pub queue_name: String,
  user: String,
  messages: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MqChannel {
  pub fn connect(login: MqLogin, consumer: Arc<RwLock<HashMap<String, Vec<u8>>>>) -> MqChannel {
    let conn = match Connection::connect(
      &format!(
        "amqp://{}:{}@{}:{}/%2f",
        login.user, login.password, login.host, login.port
      ),
      ConnectionProperties::default(),
    )
    .wait()
    {
      Ok(conn) => conn,
      Err(e) => panic!("{} unable to connect to rabbitmq server. Error: {}", ERROR, e),
    };
    let channel = match conn.create_channel().wait() {
      Ok(channel) => channel,
      Err(e) => panic!("{} unable to open rabbitmq channel. Error: {}", ERROR, e),
    };
    MqChannel {
      channel,
      queue_name: "".to_string(),
      user: login.user,
      messages: consumer,
    }
  }
  pub fn register_exchange(&mut self, exchange_name: &str) {
    let _ = match self
      .channel
      .exchange_declare(
        exchange_name,
        ExchangeKind::Topic,
        ExchangeDeclareOptions {
          passive: false,
          durable: true,
          auto_delete: true,
          internal: false,
          nowait: false,
        },
        FieldTable::default(),
      )
      .wait()
    {
      Ok(_) => info!("successfully declared exchange {}", &exchange_name),
      Err(e) => panic!("{} unable to declare rabbitmq exchange: {}", ERROR, e),
    };
  }
  pub fn register_queue(&mut self, queue_name: &str) {
    let _ = match self
      .channel
      .queue_declare(queue_name, QueueDeclareOptions::default(), FieldTable::default())
      .wait()
    {
      Ok(_) => info!("successfully declared queue {}", queue_name),
      Err(e) => panic!("{} unable to declare rabbitmq queue. Error: {}", ERROR, e),
    };
    self.queue_name = queue_name.to_string()
  }
  pub fn bind_queue(&mut self, queue_name: &str, exchange_name: &str, routing_key: &str) {
    match &self
      .channel
      .queue_bind(
        queue_name,
        exchange_name,
        routing_key,
        QueueBindOptions::default(),
        FieldTable::default(),
      )
      .wait()
    {
      Ok(_) => info!(
        "successfully binded key: {} on exchange: {} to queue: {} to ",
        routing_key, exchange_name, queue_name
      ),
      Err(e) => panic!("{} unable to declare rabbitmq queue. Error: {}", ERROR, e),
    };
  }
  pub fn set_qos(&mut self, prefetch: u16) {
    let _ = match &self.channel.basic_qos(prefetch, BasicQosOptions::default()).wait() {
      Ok(_) => info!("registered QOS"),
      Err(e) => panic!("{} unable to register queue consumers. Error: {}", ERROR, e),
    };
  }
  #[allow(dead_code)]
  pub async fn start_consuming(&mut self) {
    let consumer = match self
      .channel
      .basic_consume(
        &self.queue_name,
        "my_consumer",
        BasicConsumeOptions {
          no_local: false,
          no_ack: false,
          exclusive: false,
          nowait: false,
        },
        FieldTable::default(),
      )
      .wait()
    {
      Ok(consumer) => consumer,
      Err(e) => panic!("{} unable to register queue consumers. Error: {}", ERROR, e),
    };

    while let Some(msg) = consumer.clone().into_iter().next() {
      let (_, msg) = msg.expect("error in consumer");
      let cid = msg.properties.correlation_id().clone().unwrap();
      debug!("received message {}", cid.as_str());
      self.insert_msg_to_local_queue(cid.as_str(), msg.data.clone());
      let _ = msg.ack(BasicAckOptions::default()).await;
    }
  }
  #[allow(dead_code)]
  pub fn consume(&mut self) -> Consumer {
    let _ = match &self.channel.basic_qos(10, BasicQosOptions::default()).wait() {
      Ok(_) => info!("registered QOS"),
      Err(e) => panic!("{} unable to register queue consumers. Error: {}", ERROR, e),
    };
    let consumer = match self
      .channel
      .basic_consume(
        &self.queue_name,
        "my_consumer",
        BasicConsumeOptions {
          no_local: false,
          no_ack: false,
          exclusive: false,
          nowait: false,
        },
        FieldTable::default(),
      )
      .wait()
    {
      Ok(consumer) => consumer,
      Err(e) => panic!("{} unable to register queue consumers. Error: {}", ERROR, e),
    };
    consumer
  }
  #[allow(dead_code)]
  pub async fn request_reply_with_timeout(mut self, key: &str, data: Vec<u8>, timeout: u64) -> Result<Vec<u8>, String> {
    let now = SystemTime::now();
    let id = Uuid::new_v4();
    let correlation_id = ShortString::from(id.to_string());
    debug!("ID: {}", id);
    let _ = self
      .channel
      .basic_publish(
        "service",
        key,
        BasicPublishOptions::default(),
        data.to_vec(),
        BasicProperties::default()
          .with_user_id(ShortString::from(self.user.clone()))
          .with_correlation_id(correlation_id)
          .with_expiration(ShortString::from(MSG_EXPIRATION.to_string()))
          .with_reply_to(ShortString::from(self.queue_name.clone())),
      )
      .wait();

    loop {
      match now.elapsed() {
        Ok(elapsed) => {
          if elapsed.as_secs() > timeout {
            break;
          }
        }
        Err(e) => {
          error!("{} retrieving elapsed time: {:?}", ERROR, e);
          continue;
        }
      };
      if !self.find_msg_in_local_queue(&id.to_string()) {
        continue;
      };

      match self.get_msg_from_local_queue(&id.to_string()) {
        Ok(v) => return Ok(v),
        Err(_e) => return Err(format!("{} failed to retrieve key {}", ERROR, id)),
      };
    }
    Err(format!(
      "{} timeout while retrieving data from {} after {}s",
      ERROR, key, timeout,
    ))
  }
  #[allow(dead_code)]
  pub async fn reply(&mut self, prop: BasicProperties, data: Vec<u8>) {
    let _ = self
      .channel
      .basic_publish(
        "service",
        prop.reply_to().clone().unwrap().as_str(),
        BasicPublishOptions::default(),
        data.to_vec(),
        BasicProperties::default()
          .with_user_id(ShortString::from(self.user.clone()))
          .with_expiration(ShortString::from(MSG_EXPIRATION.to_string()))
          .with_correlation_id(prop.correlation_id().clone().unwrap()),
      )
      .wait();
  }
  #[allow(dead_code)]
  fn insert_msg_to_local_queue(&mut self, id: &str, value: Vec<u8>) {
    let data = Arc::clone(&self.messages);
    let mut data = data.write().unwrap();
    data.insert(id.to_string(), value);
    drop(data);
  }
  #[allow(dead_code)]
  fn find_msg_in_local_queue(&mut self, id: &str) -> bool {
    let data = Arc::clone(&self.messages);
    let data = data.read().unwrap();
    data.clone().contains_key(id)
  }
  #[allow(dead_code)]
  fn get_msg_from_local_queue(&mut self, id: &str) -> Result<Vec<u8>, String> {
    let data = Arc::clone(&self.messages);
    let mut data = data.write().unwrap();
    let result = match data.clone().get(id) {
      Some(d) => d.to_vec(),
      None => {
        drop(data);
        return Err(format!("{} failed to get entry by id {}", ERROR, id));
      }
    };
    data.remove(id);
    drop(data);
    Ok(result)
  }
}
