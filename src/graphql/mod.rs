use diesel::{
  backend::Backend,
  r2d2::{ConnectionManager, PooledConnection},
  Connection,
};
use juniper::LookAheadSelection;
use wundergraph::{
  error::Result,
  query_builder::selection::{offset::ApplyOffset, BoxedQuery, LoadingHandler, QueryModifier},
  scalar::WundergraphScalarValue,
  WundergraphContext,
};

use super::db::DBConnection;

pub mod mutations;
pub mod query;

#[derive(Debug)]
pub struct GQLContext<Conn>
where
  Conn: Connection + 'static,
{
  conn: PooledConnection<ConnectionManager<Conn>>,
  pub user_id: Option<i32>,
}

impl<Conn> GQLContext<Conn>
where
  Conn: Connection + 'static,
{
  pub fn new(conn: PooledConnection<ConnectionManager<Conn>>, user_id: Option<i32>) -> Self {
    Self { conn, user_id }
  }
}

impl<T, C, DB> QueryModifier<T, DB> for GQLContext<C>
where
  C: Connection<Backend = DB>,
  DB: Backend + ApplyOffset + 'static,
  T: LoadingHandler<DB, Self>,
  Self: WundergraphContext,
  Self::Connection: Connection<Backend = DB>,
{
  fn modify_query<'a>(
    &self,
    _select: &LookAheadSelection<'_, WundergraphScalarValue>,
    query: BoxedQuery<'a, T, DB, Self>,
  ) -> Result<BoxedQuery<'a, T, DB, Self>> {
    match T::TYPE_NAME {
      _ => Ok(query),
    }
  }
}

impl WundergraphContext for GQLContext<DBConnection> {
  type Connection = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DBConnection>>;

  fn get_connection(&self) -> &Self::Connection {
    &self.conn
  }
}

impl juniper::Context for GQLContext<DBConnection> {}

pub type Schema<Ctx> =
  juniper::RootNode<'static, query::Query<Ctx>, mutations::Mutation<Ctx>, WundergraphScalarValue>;
