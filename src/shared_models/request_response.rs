use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
  pub request_user: String,
  pub model: String,
  pub method: String,
  pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
  pub response_user: String,
  pub payload: Vec<u8>,
  pub error: Option<String>,
}
