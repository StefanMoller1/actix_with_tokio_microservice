use super::super::AppState;
use super::{get_user_session, set_user_session, Cache};

use actix_session::Session;
use actix_web::{error, web, web::Data, Error, HttpResponse, Result};
use log::{debug, error, info};
use std::collections::HashMap;

// store tera template in application state
pub async fn file(
  state: Data<AppState>,
  query: web::Query<HashMap<String, String>>,
  session: Session,
) -> Result<HttpResponse, Error> {
  match get_user_session(&session) {
    Some(user_cache) => {
      info!("{}", user_cache)
    }
    None => {
      info!("no session");
      let _s = Cache {
        user_id: "1234567890".to_string(),
        company_id: "qwertyuiop".to_string(),
      };
      match set_user_session(session, _s) {
        Ok(_) => debug!("session saved"),
        Err(e) => error!("failed to save session {}", e),
      };
    }
  };
  let s = if let Some(name) = query.get("name") {
    let mut ctx = tera::Context::new();
    ctx.insert("name", &name.to_owned());
    ctx.insert("text", &"Welcome!".to_owned());
    state
      .tmpl
      .render("user.html", &ctx)
      .map_err(|_| error::ErrorInternalServerError("Template error"))?
  } else {
    state
      .tmpl
      .render("index.html", &tera::Context::new())
      .map_err(|_| error::ErrorInternalServerError("Template error"))?
  };
  Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
