use super::super::AppState;
use super::file_server;
use crate::shared_models::request_response::{Request, Response};
use actix_web::{web, HttpResponse, Responder};
#[allow(unused_imports)]
use log::{debug, error, info};
use std::str;

pub fn dispatcher(app: &mut web::ServiceConfig) {
  app.route("/curl/{id}", web::get().to(curl_test));
  app.route("/", web::get().to(file_server::file));
}

async fn curl_test(web::Path(id): web::Path<String>, state: web::Data<AppState>) -> impl Responder {
  debug!("model: {:?}", &id);
  state.metrics.with_label_values(&["endpoint", "method", "status"]).inc();
  let _req = Request {
    request_user: String::from("api_service"),
    model: String::from("backend"),
    method: String::from("fetch"),
    payload: id.as_bytes().to_vec(),
  };
  let req: Vec<u8> = bincode::serialize(&_req).unwrap();
  match state
    .mq
    .clone()
    .request_reply_with_timeout("backend.request", req, 20)
    .await
  {
    Ok(data) => {
      let res: Response = bincode::deserialize(&data).unwrap();
      info!("{:?}", res);
      HttpResponse::Ok().json(str::from_utf8(&res.payload).unwrap())
    }
    Err(e) => {
      error!("failed to erecute rpc, {}", e);
      HttpResponse::Ok().json("error")
    }
  }
}
