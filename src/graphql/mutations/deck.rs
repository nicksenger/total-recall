use diesel::{pg::Pg, prelude::*};
use juniper::*;
use wundergraph::{
  query_builder::{
    mutations::{DeletedCount, HandleBatchInsert, HandleDelete, HandleInsert, HandleUpdate},
    selection::LoadingHandler,
  },
  scalar::WundergraphScalarValue,
  WundergraphContext,
};

use crate::{
  db::{schema::decks, DBConnection},
  graphql::{query::Deck, GQLContext},
  TRCError,
};

#[derive(GraphQLInputObject, Clone, Debug)]
pub struct NewDeck {
  name: String,
  language: i32,
}

impl HandleInsert<Deck, NewDeck, Pg, GQLContext<DBConnection>> for decks::table {
  fn handle_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: NewDeck,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let look_ahead = executor.look_ahead();
      let inserted = diesel::insert_into(decks::table)
        .values((
          decks::name.eq(insertable.name),
          decks::owner.eq(id),
          decks::language.eq(insertable.language),
        ))
        .returning(decks::id)
        .get_result::<i32>(conn)?;
      let query = <Deck as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(decks::id.eq(inserted));
      let items = Deck::load(&look_ahead, selection, executor, query)?;
      Ok(items.into_iter().next().unwrap_or(Value::Null))
    })
  }
}

impl HandleBatchInsert<Deck, NewDeck, Pg, GQLContext<DBConnection>> for decks::table {
  fn handle_batch_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: Vec<NewDeck>,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let look_ahead = executor.look_ahead();
      let insert = insertable
        .into_iter()
        .map(|NewDeck { name, language }| {
          (
            decks::name.eq(name),
            decks::owner.eq(id),
            decks::language.eq(language),
          )
        })
        .collect::<Vec<_>>();
      let inserted = diesel::insert_into(decks::table)
        .values(insert)
        .returning(decks::id)
        .get_results::<i32>(conn)?;

      let query = <Deck as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(decks::id.eq_any(inserted));
      let items = Deck::load(&look_ahead, selection, executor, query)?;
      Ok(Value::list(items))
    })
  }
}

#[derive(Identifiable, GraphQLInputObject, Debug)]
#[table_name = "decks"]
pub struct DeckChangeset {
  id: i32,
  name: String,
}

impl HandleUpdate<Deck, DeckChangeset, Pg, GQLContext<DBConnection>> for decks::table {
  fn handle_update(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    update: &DeckChangeset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let owner_id = decks::table
        .select(decks::owner)
        .filter(decks::id.eq(update.id))
        .get_result::<i32>(conn)?;

      if id != owner_id {
        return ExecutionResult::from(TRCError::Unauthorized);
      };

      diesel::update(decks::table.filter(decks::id.eq(update.id)))
        .set(decks::name.eq(&update.name))
        .execute(conn)?;

      let look_ahead = executor.look_ahead();

      let query = <Deck as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(decks::id.eq(update.id));
      let items = Deck::load(&look_ahead, selection, executor, query)?;
      Ok(items.into_iter().next().unwrap_or(Value::Null))
    })
  }
}

#[derive(GraphQLInputObject, Debug)]
pub struct DeckDeleteset {
  id: i32,
}

impl HandleDelete<Deck, DeckDeleteset, Pg, GQLContext<DBConnection>> for decks::table {
  fn handle_delete(
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    to_delete: &DeckDeleteset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let target_user_id = decks::table
        .select(decks::owner)
        .filter(decks::id.eq(to_delete.id))
        .get_result::<i32>(conn)?;

      if id != target_user_id {
        return ExecutionResult::from(TRCError::Unauthorized);
      };

      let d = diesel::delete(decks::table.filter(decks::id.eq(to_delete.id)));
      executor.resolve_with_ctx(
        &(),
        &DeletedCount {
          count: d.execute(conn)? as _,
        },
      )
    })
  }
}
