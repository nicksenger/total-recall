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

  use crate::test::{CreateDeckResponse, LoginResponse};

  #[actix_rt::test]
  async fn test_deck() {
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
            name
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
    let create_response: CreateDeckResponse = serde_json::from_str(&body).unwrap();

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation UpdateDeck($id: Int!, $name: String!) {
          UpdateDeck(UpdateDeck: { name: $name, id: $id }) {
            name
          }
        }",
        "variables": {
          "name": "changed_name",
          "id": create_response.data.CreateDeck.id,
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to update deck");

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "query GetDeck($id: Int!) {
          Deck(primaryKey: { id: $id }) {
            name
          }
        }",
        "variables": {
          "id": create_response.data.CreateDeck.id,
        },
      }))
      .header("Authorization", login_response.token.clone())
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to read deck");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"Deck\":{\"name\":\"changed_name\"}}}"
    );

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation DeleteDeck($id: Int!) {
          DeleteDeck(DeleteDeck: { id: $id }) {
            count
          }
        }",
        "variables": {
          "id": &create_response.data.CreateDeck.id,
        },
      }))
      .header("Authorization", login_response.token)
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to delete deck");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"DeleteDeck\":{\"count\":1}}}"
    );
  }
}
