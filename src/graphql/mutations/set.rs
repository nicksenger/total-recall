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
  db::{
    schema::{set_cards, sets},
    DBConnection,
  },
  graphql::{query::Set, GQLContext},
};

#[derive(GraphQLInputObject, Clone, Debug)]
pub struct NewSet {
  name: String,
  deck: i32,
  cards: Vec<i32>,
}

impl HandleInsert<Set, NewSet, Pg, GQLContext<DBConnection>> for sets::table {
  fn handle_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: NewSet,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let look_ahead = executor.look_ahead();
        let inserted = diesel::insert_into(sets::table)
          .values((
            sets::name.eq(insertable.name),
            sets::deck.eq(insertable.deck),
            sets::owner.eq(id),
            sets::created_at.eq(time),
          ))
          .returning(sets::id)
          .get_result::<i32>(conn)?;

        diesel::insert_into(set_cards::table)
          .values(
            insertable
              .cards
              .into_iter()
              .map(|card_id| {
                (
                  set_cards::card_id.eq(card_id),
                  set_cards::set_id.eq(inserted),
                )
              })
              .collect::<Vec<_>>(),
          )
          .returning(set_cards::id)
          .get_results::<i32>(conn)?;

        let query = <Set as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(sets::id.eq(inserted));
        let items = Set::load(&look_ahead, selection, executor, query)?;
        Ok(items.into_iter().next().unwrap_or(Value::Null))
      }
      None => Err(FieldError::new(
        "You must be logged in to create a set.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}

impl HandleBatchInsert<Set, NewSet, Pg, GQLContext<DBConnection>> for sets::table {
  fn handle_batch_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: Vec<NewSet>,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let look_ahead = executor.look_ahead();
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let mut card_ids = vec![];
        let insert = insertable
          .into_iter()
          .map(
            |NewSet {
               name,
               deck,
               cards,
             }| {
              card_ids.push(cards);
              (
                sets::name.eq(name),
                sets::deck.eq(deck),
                sets::owner.eq(id),
                sets::created_at.eq(time),
              )
            },
          )
          .collect::<Vec<_>>();

        let inserted = diesel::insert_into(sets::table)
          .values(insert)
          .returning(sets::id)
          .get_results::<i32>(conn)?;

        for (i, ids) in card_ids.iter().enumerate() {
          diesel::insert_into(set_cards::table)
            .values(
              ids
                .into_iter()
                .map(|card_id| {
                  (
                    set_cards::card_id.eq(card_id),
                    set_cards::set_id.eq(inserted[i]),
                  )
                })
                .collect::<Vec<_>>(),
            )
            .returning(set_cards::id)
            .get_results::<i32>(conn)?;
        }

        let query = <Set as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(sets::id.eq_any(inserted));
        let items = Set::load(&look_ahead, selection, executor, query)?;
        Ok(Value::list(items))
      }
      None => Err(FieldError::new(
        "You must be logged in to create sets.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}

#[derive(GraphQLInputObject, Identifiable, Debug)]
#[table_name = "sets"]
pub struct SetChangeset {
  id: i32,
  name: String,
}

impl HandleUpdate<Set, SetChangeset, Pg, GQLContext<DBConnection>> for sets::table {
  fn handle_update(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    update: &SetChangeset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let owner_id = sets::table
          .filter(sets::id.eq(update.id))
          .select(sets::id)
          .get_result::<i32>(conn)?;

        if id != owner_id {
          return Err(FieldError::new(
            "Updating other users' sets is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        diesel::update(sets::table.filter(sets::id.eq(update.id)))
          .set(sets::name.eq(&update.name))
          .execute(conn)?;

        let look_ahead = executor.look_ahead();

        let query = <Set as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(sets::id.eq(update.id));
        let items = Set::load(&look_ahead, selection, executor, query)?;
        Ok(items.into_iter().next().unwrap_or(Value::Null))
      }
      None => Err(FieldError::new(
        "You must be logged in to update a set.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}

#[derive(GraphQLInputObject, Debug)]
pub struct SetDeleteset {
  id: i32,
}

impl HandleDelete<Set, SetDeleteset, Pg, GQLContext<DBConnection>> for sets::table {
  fn handle_delete(
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    to_delete: &SetDeleteset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let target_user_id = sets::table
          .filter(sets::id.eq(to_delete.id))
          .select(sets::owner)
          .get_result::<i32>(conn)?;

        if id != target_user_id {
          return Err(FieldError::new(
            "Deleting other users' sets is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        let d = diesel::delete(sets::table.filter(sets::id.eq(to_delete.id)));
        executor.resolve_with_ctx(
          &(),
          &DeletedCount {
            count: d.execute(conn)? as _,
          },
        )
      }
      None => Err(FieldError::new(
        "You must be logged in to delete a set.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}
