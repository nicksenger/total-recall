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

  use crate::test::{CreateCardResponse, CreateDeckResponse, LoginResponse};

  #[actix_rt::test]
  async fn test_card() {
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
        "query": "mutation UpdateCard($id: Int!, $link: String!) {
          UpdateCard(UpdateCard: { id: $id, link: $link }) {
            link
          }
        }",
        "variables": {
          "id": create_card_response.data.CreateCard.id,
          "link": "http://foo.com/bar/baz"
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to update card");

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "query GetCard($id: Int!) {
          Card(primaryKey: { id: $id }) {
            link
          }
        }",
        "variables": {
          "id": create_card_response.data.CreateCard.id,
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to read card");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"Card\":{\"link\":\"http://foo.com/bar/baz\"}}}"
    );

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation DeleteCard($id: Int!) {
          DeleteCard(DeleteCard: { id: $id }) {
            count
          }
        }",
        "variables": {
          "id": &create_card_response.data.CreateCard.id,
        },
      }))
      .header("Authorization", login_response.token)
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to delete card");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"DeleteCard\":{\"count\":1}}}"
    );
  }
}
