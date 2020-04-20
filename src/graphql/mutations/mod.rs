use super::query::{Card, Deck, Score, Set, User};

mod card;
mod deck;
mod score;
mod set;
mod user;
mod utilities;

use card::{CardChangeset, CardDeleteset, NewCard};
use deck::{DeckChangeset, DeckDeleteset, NewDeck};
use score::{NewScore, ScoreChangeset};
use set::{NewSet, SetChangeset, SetDeleteset};
use user::{NewUser, UserChangeset, UserDeleteset};

wundergraph::mutation_object! {
Mutation {
  User(insert = NewUser, update = UserChangeset, delete = UserDeleteset),
  Deck(insert = NewDeck, update = DeckChangeset, delete = DeckDeleteset),
  Card(insert = NewCard, update = CardChangeset, delete = CardDeleteset),
  Score(insert = NewScore, update = ScoreChangeset, delete = false),
  Set(insert = NewSet, update = SetChangeset, delete = SetDeleteset),
}
}
