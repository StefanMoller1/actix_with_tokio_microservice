use actix_session::Session;
use actix_web::Error;
#[allow(unused_imports)]
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{fmt, str};

mod file_server;
pub mod routes;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Cache {
  user_id: String,
  company_id: String,
}

impl fmt::Display for Cache {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "user_id: {}, company_id: {}", self.user_id, self.company_id)
  }
}

fn get_user_session(session: &Session) -> Option<Cache> {
  match session.get::<Vec<u8>>("session") {
    Ok(data) => data.map(|x| bincode::deserialize(&x).unwrap()),
    Err(e) => {
      error!("error retrieving cache value {}", e);
      None
    }
  }
}

fn set_user_session(session: Session, data: Cache) -> Result<(), Error> {
  let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
  session.set("session", encoded)
}
