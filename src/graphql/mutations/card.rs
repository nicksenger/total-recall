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
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let (abbr, (target_user_id, language)) = decks::table
          .inner_join(languages::table)
          .select((languages::abbreviation, (decks::owner, decks::language)))
          .filter(decks::id.eq(insertable.deck))
          .get_result::<(String, (i32, i32))>(conn)?;

        if id != target_user_id {
          return Err(FieldError::new(
            "Creating cards for other users is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        }
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let look_ahead = executor.look_ahead();

        let sanitized = sanitize(&insertable.back);
        match get_audio_from_google(&abbr, &insertable.back, &sanitized) {
          Ok(_) => match get_image_from_google(&abbr, &insertable.back, &sanitized) {
            Ok(_) => {
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
            }
            _ => Err(FieldError::new(
              "Failed retrieve card image.",
              graphql_value!({
                  "type": "INTERNAL"
              }),
            )),
          },
          _ => Err(FieldError::new(
            "Failed retrieve card audio.",
            graphql_value!({
                "type": "INTERNAL"
            }),
          )),
        }
      }
      None => Err(FieldError::new(
        "You must be logged in to create a card.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
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
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let look_ahead = executor.look_ahead();

        let mut deck_ids = vec![];
        for card in &insertable {
          deck_ids.push(card.deck);
        }

        let deck_owners_languages = decks::table
          .inner_join(languages::table)
          .select((languages::abbreviation, (decks::owner, decks::language)))
          .filter(decks::id.eq_any(deck_ids))
          .get_results::<(String, (i32, i32))>(conn)?;

        for (_, (owner, _)) in &deck_owners_languages {
          if &id != owner {
            return Err(FieldError::new(
              "Creating cards for other users is forbidden.",
              graphql_value!({
                  "type": "UNAUTHORIZED"
              }),
            ));
          }
        }

        let mut failed = false;
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let insert = insertable
          .into_iter()
          .enumerate()
          .map(
            |(
              i,
              NewCard {
                front,
                deck,
                link,
                back,
              },
            )| {
              let sanitized = sanitize(&back);
              let abbr = &deck_owners_languages[i].0;
              let inserted_back = match get_audio_from_google(abbr, &back, &sanitized) {
                Ok(_) => match get_image_from_google(abbr, &back, &sanitized) {
                  Ok(_) => {
                    match diesel::insert_into(backs::table)
                      .values((
                        backs::text.eq(back),
                        backs::language.eq((deck_owners_languages[i].1).1),
                        backs::image.eq(Some(format!("images/{}/{}.jpg", abbr, &sanitized))),
                        backs::audio.eq(Some(format!("audio/{}/{}.mp3", abbr, &sanitized))),
                      ))
                      .returning(backs::id)
                      .get_result::<i32>(conn)
                    {
                      Ok(i) => i,
                      Err(_) => {
                        failed = true;
                        0
                      }
                    }
                  }
                  _ => {
                    failed = true;
                    0
                  }
                },
                _ => {
                  failed = true;
                  0
                }
              };

              (
                cards::front.eq(front),
                cards::deck.eq(deck),
                cards::link.eq(link),
                cards::created_at.eq(time),
                cards::back.eq(inserted_back),
              )
            },
          )
          .collect::<Vec<_>>();
        if failed {
          return Err(FieldError::new(
            "Error adding card.",
            graphql_value!({
                "type": "INTERNAL"
            }),
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
      }
      None => Err(FieldError::new(
        "You must be logged in to create cards.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
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
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let (owner_id, current_link) = cards::table
          .filter(cards::id.eq(update.id))
          .inner_join(decks::table)
          .select((decks::owner, cards::link))
          .get_result::<(i32, Option<String>)>(conn)?;

        if id != owner_id {
          return Err(FieldError::new(
            "Updating other users' cards is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        diesel::update(cards::table.filter(cards::id.eq(update.id)))
          .set(cards::link.eq(update.link.as_ref().or(current_link.as_ref())))
          .execute(conn)?;

        let look_ahead = executor.look_ahead();

        let query = <Card as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(cards::id.eq(update.id));
        let items = Card::load(&look_ahead, selection, executor, query)?;
        Ok(items.into_iter().next().unwrap_or(Value::Null))
      }
      None => Err(FieldError::new(
        "You must be logged in to update a card.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
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
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let target_user_id = cards::table
          .filter(cards::id.eq(to_delete.id))
          .inner_join(decks::table)
          .select(decks::owner)
          .get_result::<i32>(conn)?;

        if id != target_user_id {
          return Err(FieldError::new(
            "Deleting other users' cards is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        let d = diesel::delete(cards::table.filter(cards::id.eq(to_delete.id)));
        executor.resolve_with_ctx(
          &(),
          &DeletedCount {
            count: d.execute(conn)? as _,
          },
        )
      }
      None => Err(FieldError::new(
        "You must be logged in to delete a card.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}
