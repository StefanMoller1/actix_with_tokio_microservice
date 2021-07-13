use std::collections::HashSet;
mod config;
pub mod run;

// Define Message keys
const BACKEND_REQUEST: &str = "backend.request";

const SERVICEKEYS: [&str; 1] = [BACKEND_REQUEST];

pub fn get_service_keys() -> HashSet<String> {
  let mut service_keys: HashSet<String> = HashSet::new();
  for key in SERVICEKEYS.iter() {
    service_keys.insert(key.to_string());
  }
  service_keys
}
