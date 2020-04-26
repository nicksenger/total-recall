use diesel::{
  prelude::*,
  r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use std::{env, sync::Arc};

use crate::{
  db::DBConnection,
  graphql::{mutations::Mutation, query::Query, GQLContext, Schema},
  service::AppState,
};

mod user;
mod deck;

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
