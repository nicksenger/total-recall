use diesel::{
  backend::Backend,
  deserialize::{self, FromSql},
  serialize::{self, ToSql},
  sql_types::SmallInt,
  Identifiable,
};
use std::io::Write;
use wundergraph::{
  query_builder::types::{HasMany, HasOne, WundergraphValue},
  WundergraphEntity,
};

use crate::db::schema::*;

#[derive(
  Debug, Copy, Clone, AsExpression, FromSqlRow, GraphQLEnum, WundergraphValue, Eq, PartialEq, Hash,
)]
#[sql_type = "SmallInt"]
pub enum ScoreValue {
  ZERO = 0,
  ONE = 1,
  TWO = 2,
  THREE = 3,
  FOUR = 4,
  FIVE = 5,
}

impl<DB> ToSql<SmallInt, DB> for ScoreValue
where
  DB: Backend,
  i16: ToSql<SmallInt, DB>,
{
  fn to_sql<W: Write>(&self, out: &mut serialize::Output<'_, W, DB>) -> serialize::Result {
    (*self as i16).to_sql(out)
  }
}

impl<DB> FromSql<SmallInt, DB> for ScoreValue
where
  DB: Backend,
  i16: FromSql<SmallInt, DB>,
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
    let value = i16::from_sql(bytes)?;
    Ok(match value {
      0 => ScoreValue::ZERO,
      1 => ScoreValue::ONE,
      2 => ScoreValue::TWO,
      3 => ScoreValue::THREE,
      4 => ScoreValue::FOUR,
      5 => ScoreValue::FIVE,
      _ => unreachable!(),
    })
  }
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "users"]
pub struct User {
  pub id: i32,
  pub username: String,
  created_at: i64,
  updated_at: i64,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "languages"]
pub struct Language {
  id: i32,
  name: String,
  abbreviation: String,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "decks"]
pub struct Deck {
  id: i32,
  name: String,
  owner: HasOne<i32, User>,
  language: HasOne<i32, Language>,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "cards"]
pub struct Card {
  id: i32,
  created_at: i64,
  front: String,
  back: HasOne<i32, Back>,
  deck: HasOne<i32, Deck>,
  link: Option<String>,
  sets: HasMany<SetCard, set_cards::card_id>,
  scores: HasMany<Score, scores::card>,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "scores"]
pub struct Score {
  id: i32,
  created_at: i64,
  card: HasOne<i32, Card>,
  value: ScoreValue,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "backs"]
pub struct Back {
  id: i32,
  text: String,
  language: HasOne<i32, Language>,
  audio: Option<String>,
  image: Option<String>,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "sets"]
pub struct Set {
  id: i32,
  created_at: i64,
  name: String,
  deck: HasOne<i32, Deck>,
  owner: HasOne<i32, User>,
  cards: HasMany<SetCard, set_cards::set_id>,
}

#[derive(Clone, Debug, Identifiable, Queryable, WundergraphEntity)]
#[primary_key(id)]
#[table_name = "set_cards"]
pub struct SetCard {
  #[wundergraph(skip)]
  id: i32,
  card_id: HasOne<i32, Card>,
  set_id: HasOne<i32, Set>,
}

wundergraph::query_object! {
  Query {
    User,
    Language,
    Deck,
    Card,
    Score,
    Back,
    Set,
    SetCard,
  }
}
