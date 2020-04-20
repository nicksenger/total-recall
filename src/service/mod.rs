use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;

use super::{
  db::DBConnection,
  graphql::{GQLContext, Schema},
};

pub mod endpoints;
pub mod jwt;

#[derive(Clone)]
pub struct AppState {
  pub schema: Arc<Schema<GQLContext<DBConnection>>>,
  pub pool: Arc<Pool<ConnectionManager<DBConnection>>>,
}
