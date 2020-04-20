use actix_web::{
  web::{Data, Json},
  HttpRequest, HttpResponse,
};
use bcrypt::verify;
use diesel::prelude::*;
use failure::Error;
use juniper::{graphiql::graphiql_source, http::GraphQLRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wundergraph::scalar::WundergraphScalarValue;

use crate::{
  db::schema::users,
  graphql::GQLContext,
  service::{
    jwt::{encode_jwt, verify_jwt, LoginAttempt},
    AppState,
  },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphQLData(GraphQLRequest<WundergraphScalarValue>);

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginData(LoginAttempt);

pub async fn graphiql() -> HttpResponse {
  let html = graphiql_source("/graphql");
  HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(html)
}

pub async fn graphql(
  req: HttpRequest,
  Json(GraphQLData(data)): Json<GraphQLData>,
  st: Data<AppState>,
) -> Result<HttpResponse, Error> {
  let conn = st.get_ref().pool.get()?;

  let user_id = match req.headers().get("Authorization") {
    Some(header) => match header.to_str() {
      Ok(auth) => match verify_jwt(String::from(auth)) {
        Ok(t) => Some(t.claims.user_id),
        Err(_) => None,
      },
      Err(_) => None,
    },
    None => None,
  };

  let ctx = GQLContext::new(conn, user_id);
  let res = data.execute(&st.get_ref().schema, &ctx);
  Ok(
    HttpResponse::Ok()
      .content_type("application/json")
      .body(serde_json::to_string(&res)?),
  )
}

pub async fn login(
  Json(LoginData(attempt)): Json<LoginData>,
  st: Data<AppState>,
) -> Result<HttpResponse, Error> {
  let conn = st.get_ref().pool.get()?;
  let err = Ok(
    HttpResponse::Unauthorized()
      .content_type("application/json")
      .body(json!({ "error": "Login failed" })),
  );
  match users::table
    .select((users::id, users::password))
    .filter(users::username.eq(attempt.username))
    .get_result::<(i32, String)>(&conn)
  {
    Ok((id, password)) => match verify(attempt.password, &password) {
      Ok(valid) => {
        if !valid {
          return err;
        }
        match encode_jwt(id, 30) {
          Ok(t) => Ok(
            HttpResponse::Ok()
              .content_type("application/json")
              .body(json!({ "token": t, "user_id": id })),
          ),
          Err(_) => err,
        }
      }
      Err(_) => err,
    },
    Err(_) => err,
  }
}
