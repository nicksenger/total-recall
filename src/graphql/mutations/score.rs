use diesel::{pg::Pg, prelude::*};
use juniper::*;
use std::time::{SystemTime, UNIX_EPOCH};
use wundergraph::{
  query_builder::{
    mutations::{HandleBatchInsert, HandleInsert, HandleUpdate},
    selection::LoadingHandler,
  },
  scalar::WundergraphScalarValue,
  WundergraphContext,
};

use crate::{
  db::{
    schema::{cards, decks, scores},
    DBConnection,
  },
  graphql::{
    query::{Score, ScoreValue},
    GQLContext,
  },
};

#[derive(GraphQLInputObject, Clone, Debug)]
pub struct NewScore {
  card: i32,
  value: ScoreValue,
}

impl HandleInsert<Score, NewScore, Pg, GQLContext<DBConnection>> for scores::table {
  fn handle_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: NewScore,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let target_user_id = cards::table
          .filter(cards::id.eq(insertable.card))
          .inner_join(decks::table)
          .select(decks::owner)
          .get_result::<i32>(conn)?;

        if id != target_user_id {
          return Err(FieldError::new(
            "Studying other users' cards is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        }
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let look_ahead = executor.look_ahead();
        let inserted = diesel::insert_into(scores::table)
          .values((
            scores::card.eq(insertable.card),
            scores::value.eq(insertable.value),
            scores::created_at.eq(time),
          ))
          .returning(scores::id)
          .get_result::<i32>(conn)?;

        let query = <Score as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(scores::id.eq(inserted));
        let items = Score::load(&look_ahead, selection, executor, query)?;
        Ok(items.into_iter().next().unwrap_or(Value::Null))
      }
      None => Err(FieldError::new(
        "You must be logged in to study.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}

impl HandleBatchInsert<Score, NewScore, Pg, GQLContext<DBConnection>> for scores::table {
  fn handle_batch_insert(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    insertable: Vec<NewScore>,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let look_ahead = executor.look_ahead();

        let mut card_ids = vec![];
        for score in &insertable {
          card_ids.push(score.card);
        }

        let target_ids = cards::table
          .filter(cards::id.eq_any(card_ids))
          .inner_join(decks::table)
          .select(decks::owner)
          .get_results::<i32>(conn)?;

        for target_id in target_ids {
          if id != target_id {
            return Err(FieldError::new(
              "Submitting scores for other users' cards is forbidden.",
              graphql_value!({
                  "type": "UNAUTHORIZED"
              }),
            ));
          }
        }

        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let insert = insertable
          .into_iter()
          .map(|NewScore { card, value }| {
            (
              scores::card.eq(card),
              scores::value.eq(value),
              scores::created_at.eq(time),
            )
          })
          .collect::<Vec<_>>();

        let inserted = diesel::insert_into(scores::table)
          .values(insert)
          .returning(scores::id)
          .get_results::<i32>(conn)?;

        let query = <Score as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(scores::id.eq_any(inserted));
        let items = Score::load(&look_ahead, selection, executor, query)?;
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

#[derive(GraphQLInputObject, Debug)]
pub struct ScoreChangeset {
  id: i32,
  value: ScoreValue,
}

impl HandleUpdate<Score, ScoreChangeset, Pg, GQLContext<DBConnection>> for scores::table {
  fn handle_update(
    selection: Option<&'_ [Selection<'_, WundergraphScalarValue>]>,
    executor: &Executor<'_, GQLContext<DBConnection>, WundergraphScalarValue>,
    update: &ScoreChangeset,
  ) -> ExecutionResult<WundergraphScalarValue> {
    let ctx = executor.context();
    let conn = ctx.get_connection();
    conn.transaction(|| match ctx.user_id {
      Some(id) => {
        let owner_id = scores::table
          .filter(scores::id.eq(update.id))
          .inner_join(cards::table.inner_join(decks::table))
          .select(decks::owner)
          .get_result::<i32>(conn)?;

        if id != owner_id {
          return Err(FieldError::new(
            "Updating other users' scores is forbidden.",
            graphql_value!({
                "type": "UNAUTHORIZED"
            }),
          ));
        };

        diesel::update(scores::table.filter(scores::id.eq(update.id)))
          .set(scores::value.eq(update.value))
          .execute(conn)?;

        let look_ahead = executor.look_ahead();

        let query = <Score as LoadingHandler<_, PgConnection>>::build_query(&[], &look_ahead)?
          .filter(scores::id.eq(update.id));
        let items = Score::load(&look_ahead, selection, executor, query)?;
        Ok(items.into_iter().next().unwrap_or(Value::Null))
      }
      None => Err(FieldError::new(
        "You must be logged in to update a score.",
        graphql_value!({
            "type": "UNAUTHORIZED"
        }),
      )),
    })
  }
}
