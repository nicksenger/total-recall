use bcrypt::hash;
use diesel::{pg::Pg, prelude::*};
use juniper::*;
use std::time::{SystemTime, UNIX_EPOCH};
use wundergraph::{
  query_builder::{
    mutations::{DeletedCount, HandleBatchInsert, HandleDelete, HandleInsert, HandleUpdate},
    selection::LoadingHandler,
  },
  scalar::WundergraphScalarValue,
  WundergraphContext,
};

use crate::{
  db::{schema::users, DBConnection},
  graphql::{query::User, GQLContext},
};

#[derive(Debug, GraphQLInputObject)]
pub struct NewUser {
  username: String,
  password: String,
}

impl HandleInsert<User, NewUser, Pg, GQLContext<DBConnection>> for users::table {
  fn handle_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: NewUser,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
      let look_ahead = executor.look_ahead();
      let hashed = hash(&insertable.password, 10)?;
      let inserted = diesel::insert_into(users::table)
        .values((
          users::username.eq(insertable.username),
          users::password.eq(hashed),
          users::created_at.eq(time),
          users::updated_at.eq(time),
        ))
        .returning(users::id)
        .get_result::<i32>(conn)?;

      let query = <User as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(users::id.eq(inserted));
      let items = User::load(&look_ahead, selection, executor, query)?;
      Ok(items.into_iter().next().unwrap_or(Value::Null))
    })
  }
}

impl HandleBatchInsert<User, NewUser, Pg, GQLContext<DBConnection>> for users::table {
  fn handle_batch_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: Vec<NewUser>,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
      let look_ahead = executor.look_ahead();
      let insert = insertable
        .into_iter()
        .map(|NewUser { username, password }| {
          (
            users::username.eq(username),
            users::password.eq(hash(&password, 10).unwrap()),
            users::created_at.eq(time),
            users::updated_at.eq(time),
          )
        })
        .collect::<Vec<_>>();
      let inserted = diesel::insert_into(users::table)
        .values(insert)
        .returning(users::id)
        .get_results::<i32>(conn)?;

      let query = <User as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(users::id.eq_any(inserted));
      let items = User::load(&look_ahead, selection, executor, query)?;
      Ok(Value::list(items))
    })
  }
}

#[derive(GraphQLInputObject, Debug)]
pub struct UserChangeset {
  id: i32,
  password: String,
}

impl HandleUpdate<User, UserChangeset, Pg, GQLContext<DBConnection>> for users::table {
  fn handle_update(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    update: &UserChangeset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let target_user_id = users::table
          .select(users::id)
          .filter(users::id.eq(update.id))
          .get_result::<i32>(conn)?;

        if id != target_user_id {
          return Err(FieldError::new(
            "Changing other users' passwords is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        let hashed = hash(&update.password, 10)?;
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

        diesel::update(users::table.filter(users::id.eq(update.id)))
          .set((users::password.eq(&hashed), users::updated_at.eq(time)))
          .execute(conn)?;

        let look_ahead = executor.look_ahead();

        let query = <User as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(users::id.eq(id));
        let items = User::load(&look_ahead, selection, executor, query)?;
        Ok(items.into_iter().next().unwrap_or(Value::Null))
      }
      None => Err(FieldError::new(
        "You must be logged in to change your password.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}

#[derive(GraphQLInputObject, Debug)]
pub struct UserDeleteset {
  id: i32,
}

impl HandleDelete<User, UserDeleteset, Pg, GQLContext<DBConnection>> for users::table {
  fn handle_delete(
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    to_delete: &UserDeleteset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let target_user_id = users::table
          .select(users::id)
          .filter(users::id.eq(to_delete.id))
          .get_result::<i32>(conn)?;

        if id != target_user_id {
          return Err(FieldError::new(
            "Deleting other users is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        let d = diesel::delete(users::table.filter(users::id.eq(to_delete.id)));
        executor.resolve_with_ctx(
          &(),
          &DeletedCount {
            count: d.execute(conn)? as _,
          },
        )
      }
      None => Err(FieldError::new(
        "You must be logged in to close your account.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}
