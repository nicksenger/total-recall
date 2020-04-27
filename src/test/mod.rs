use diesel::{
  prelude::*,
  r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use serde::Deserialize;
use std::{env, sync::Arc};

use crate::{
  db::DBConnection,
  graphql::{mutations::Mutation, query::Query, GQLContext, Schema},
  service::AppState,
};

mod card;
mod deck;
mod score;
mod set;
mod user;

#[derive(Deserialize)]
struct LoginResponse {
  token: String,
  user_id: i32,
}

#[derive(Deserialize)]
struct CreateDeckInfo {
  id: i32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct CreateDeckData {
  CreateDeck: CreateDeckInfo,
}

#[derive(Deserialize)]
struct CreateDeckResponse {
  data: CreateDeckData,
}

#[derive(Deserialize)]
struct CreateCardInfo {
  id: i32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct CreateCardData {
  CreateCard: CreateCardInfo,
}

#[derive(Deserialize)]
struct CreateCardResponse {
  data: CreateCardData,
}

#[derive(Deserialize)]
struct CreateSetInfo {
  id: i32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct CreateSetData {
  CreateSet: CreateSetInfo,
}

#[derive(Deserialize)]
struct CreateSetResponse {
  data: CreateSetData,
}

#[derive(Deserialize)]
struct CreateScoreInfo {
  id: i32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct CreateScoreData {
  CreateScore: CreateScoreInfo,
}

#[derive(Deserialize)]
struct CreateScoreResponse {
  data: CreateScoreData,
}

#[cfg(test)]
pub fn init() -> AppState {
  dotenv().ok();
  let db_url = env::var("DATABASE_URL").expect("Database url not set");
  let manager = ConnectionManager::<DBConnection>::new(db_url);
  let pool = Pool::builder()
    .max_size(1)
    .build(manager)
    .expect("Failed to initialize connection pool");
  let conn = pool.get().expect("Failed to get db connection");
  conn
    .begin_test_transaction()
    .expect("Failed to start transaction");

  let query = Query::<GQLContext<DBConnection>>::default();
  let mutation = Mutation::<GQLContext<DBConnection>>::default();
  let schema = Schema::new(query, mutation);

  let schema = Arc::new(schema);
  let pool = Arc::new(pool);
  AppState { schema, pool }
}
