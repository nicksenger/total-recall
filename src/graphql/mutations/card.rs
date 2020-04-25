use diesel::{pg::Pg, prelude::*};
use juniper::*;
use sanitize_filename::sanitize;
use std::time::{SystemTime, UNIX_EPOCH};
use wundergraph::{
  query_builder::{
    mutations::{DeletedCount, HandleBatchInsert, HandleDelete, HandleInsert, HandleUpdate},
    selection::LoadingHandler,
  },
  scalar::WundergraphScalarValue,
  WundergraphContext,
};

use super::utilities::{get_audio_from_google, get_image_from_google};
use crate::{
  db::{
    schema::{backs, cards, decks, languages},
    DBConnection,
  },
  graphql::{query::Card, GQLContext},
  TRCError,
};

#[derive(GraphQLInputObject, Clone, Debug)]
pub struct NewCard {
  front: String,
  back: String,
  deck: i32,
  link: Option<String>,
}

impl HandleInsert<Card, NewCard, Pg, GQLContext<DBConnection>> for cards::table {
  fn handle_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: NewCard,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let (abbr, (target_user_id, language)) = decks::table
        .inner_join(languages::table)
        .select((languages::abbreviation, (decks::owner, decks::language)))
        .filter(decks::id.eq(insertable.deck))
        .get_result::<(String, (i32, i32))>(conn)?;
      if id != target_user_id {
        return ExecutionResult::from(TRCError::Unauthorized);
      }

      let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
      let look_ahead = executor.look_ahead();
      let sanitized = sanitize(&insertable.back);

      get_audio_from_google(&abbr, &insertable.back, &sanitized)?;
      get_image_from_google(&abbr, &insertable.back, &sanitized)?;

      let inserted_back = diesel::insert_into(backs::table)
        .values((
          backs::text.eq(insertable.back),
          backs::language.eq(language),
          backs::image.eq(Some(format!("images/{}/{}.jpg", &abbr, &sanitized))),
          backs::audio.eq(Some(format!("audio/{}/{}.mp3", &abbr, &sanitized))),
        ))
        .returning(backs::id)
        .get_result::<i32>(conn)?;
      let inserted = diesel::insert_into(cards::table)
        .values((
          cards::front.eq(insertable.front),
          cards::deck.eq(insertable.deck),
          cards::link.eq(insertable.link),
          cards::created_at.eq(time),
          cards::back.eq(inserted_back),
        ))
        .returning(cards::id)
        .get_result::<i32>(conn)?;
      let query = <Card as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(cards::id.eq(inserted));
      let items = Card::load(&look_ahead, selection, executor, query)?;
      Ok(items.into_iter().next().unwrap_or(Value::Null))
    })
  }
}

impl HandleBatchInsert<Card, NewCard, Pg, GQLContext<DBConnection>> for cards::table {
  fn handle_batch_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: Vec<NewCard>,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let look_ahead = executor.look_ahead();

      let mut deck_ids = vec![];
      for card in &insertable {
        deck_ids.push(card.deck)
      }

      let decks_owners_languages = decks::table
        .inner_join(languages::table)
        .select((languages::abbreviation, (decks::owner, decks::language)))
        .filter(decks::id.eq_any(deck_ids))
        .get_results::<(String, (i32, i32))>(conn)?;

      for (_, (owner, _)) in &decks_owners_languages {
        if &id != owner {
          return ExecutionResult::from(TRCError::Unauthorized);
        }
      }

      let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

      let mut insert = vec![];
      for (
        i,
        NewCard {
          front,
          deck,
          link,
          back,
        },
      ) in insertable.into_iter().enumerate()
      {
        let sanitized = sanitize(&back);
        let abbr = &decks_owners_languages[i].0;

        get_audio_from_google(abbr, &back, &sanitized)?;
        get_image_from_google(abbr, &back, &sanitized)?;
        let inserted_back = diesel::insert_into(backs::table)
          .values((
            backs::text.eq(back),
            backs::language.eq((decks_owners_languages[i].1).1),
            backs::image.eq(Some(format!("images/{}/{}.jpg", abbr, &sanitized))),
            backs::audio.eq(Some(format!("audio/{}/{}.mp3", abbr, &sanitized))),
          ))
          .returning(backs::id)
          .get_result::<i32>(conn)?;

        insert.push((
          cards::front.eq(front),
          cards::deck.eq(deck),
          cards::link.eq(link),
          cards::created_at.eq(time),
          cards::back.eq(inserted_back),
        ));
      }

      let inserted = diesel::insert_into(cards::table)
        .values(insert)
        .returning(cards::id)
        .get_results::<i32>(conn)?;

      let query = <Card as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(cards::id.eq_any(inserted));
      let items = Card::load(&look_ahead, selection, executor, query)?;
      Ok(Value::list(items))
    })
  }
}

#[derive(Identifiable, GraphQLInputObject, Debug)]
#[table_name = "cards"]
pub struct CardChangeset {
  id: i32,
  link: Option<String>,
}

impl HandleUpdate<Card, CardChangeset, Pg, GQLContext<DBConnection>> for cards::table {
  fn handle_update(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    update: &CardChangeset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let (owner_id, current_link) = cards::table
        .filter(cards::id.eq(update.id))
        .inner_join(decks::table)
        .select((decks::owner, cards::link))
        .get_result::<(i32, Option<String>)>(conn)?;

      if id != owner_id {
        return ExecutionResult::from(TRCError::Unauthorized);
      }

      diesel::update(cards::table.filter(cards::id.eq(update.id)))
        .set(cards::link.eq(update.link.as_ref().or(current_link.as_ref())))
        .execute(conn)?;

      let look_ahead = executor.look_ahead();

      let query = <Card as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
        .filter(cards::id.eq(update.id));
      let items = Card::load(&look_ahead, selection, executor, query)?;
      Ok(items.into_iter().next().unwrap_or(Value::Null))
    })
  }
}

#[derive(GraphQLInputObject, Debug)]
pub struct CardDeleteset {
  id: i32,
}

impl HandleDelete<Card, CardDeleteset, Pg, GQLContext<DBConnection>> for cards::table {
  fn handle_delete(
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    to_delete: &CardDeleteset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| {
      let id = ctx.user_id.ok_or(TRCError::Unauthorized)?;
      let target_user_id = cards::table
        .filter(cards::id.eq(to_delete.id))
        .inner_join(decks::table)
        .select(decks::owner)
        .get_result::<i32>(conn)?;

      if id != target_user_id {
        return ExecutionResult::from(TRCError::Unauthorized);
      }

      let d = diesel::delete(cards::table.filter(cards::id.eq(to_delete.id)));
      executor.resolve_with_ctx(
        &(),
        &DeletedCount {
          count: d.execute(conn)? as _,
        },
      )
    })
  }
}
