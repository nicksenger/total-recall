#[cfg(test)]
mod tests {
  use crate::{
    service::endpoints::{graphql, login},
    test::init,
  };
  use actix_web::{
    test::{self, TestRequest},
    web::post,
    App,
  };
  use serde_json::{self, json};
  use std::str::from_utf8;

  use crate::test::{CreateCardResponse, CreateDeckResponse, CreateScoreResponse, LoginResponse};

  #[actix_rt::test]
  async fn test_score() {
    let data = init();

    let mut app = test::init_service(
      App::new()
        .data(data.clone())
        .route("/login", post().to(login))
        .route("/graphql", post().to(graphql)),
    )
    .await;

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation Register($username: String!, $password: String!) {
          CreateUser(NewUser: { username: $username, password: $password }) {
            username
          }
        }",
        "variables": {
          "username": "test_user",
          "password": "test",
        },
      }))
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to create user");

    let req = TestRequest::post()
      .uri("/login")
      .set_json(&json!({
        "username": "test_user",
        "password": "test",
      }))
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Login failed");

    let body = test::read_body(resp).await;
    let body = from_utf8(&body).unwrap();
    let login_response: LoginResponse = serde_json::from_str(&body).unwrap();

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation CreateDeck($name: String!, $language: Int!) {
          CreateDeck(NewDeck: { name: $name, language: $language }) {
            id
          }
        }",
        "variables": {
          "name": "test_deck",
          "language": 1,
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to create deck");

    let body = test::read_body(resp).await;
    let body = from_utf8(&body).unwrap();
    let create_deck_response: CreateDeckResponse = serde_json::from_str(&body).unwrap();

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation CreateCard($front: String!, $back: String!, $deck: Int!) {
          CreateCard(NewCard: { deck: $deck, front: $front, back: $back }) {
            id
          }
        }",
        "variables": {
          "front": "foo",
          "back": "bar",
          "deck": create_deck_response.data.CreateDeck.id,
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to create card");

    let body = test::read_body(resp).await;
    let body = from_utf8(&body).unwrap();
    let create_card_response: CreateCardResponse = serde_json::from_str(&body).unwrap();

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation CreateScore($card: Int!, $value: ScoreValue!) {
          CreateScore(NewScore: { card: $card, value: $value }) {
            id
          }
        }",
        "variables": {
          "card": create_card_response.data.CreateCard.id,
          "value": "FIVE",
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to create score");

    let body = test::read_body(resp).await;
    let body = from_utf8(&body).unwrap();
    let create_score_response: CreateScoreResponse = serde_json::from_str(&body).unwrap();

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation UpdateScore($id: Int!, $value: ScoreValue!) {
          UpdateScore(UpdateScore: { id: $id, value: $value }) {
            value
          }
        }",
        "variables": {
          "id": create_score_response.data.CreateScore.id,
          "value": "ZERO"
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to update score");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"UpdateScore\":{\"value\":\"ZERO\"}}}"
    );

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "query GetScore($id: Int!) {
          Score(primaryKey: { id: $id }) {
            value
          }
        }",
        "variables": {
          "id": create_score_response.data.CreateScore.id,
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to read score");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"Score\":{\"value\":\"ZERO\"}}}"
    );
  }
}
