use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[allow(dead_code)]
pub fn get_time() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[allow(dead_code)]
pub fn get_time_in_msec() -> u128 {
  SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
}

#[allow(dead_code)]
pub fn new_uuid() -> String {
  Uuid::new_v4().to_string()
}
