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
  use serde::Deserialize;
  use serde_json::{self, json};
  use std::str::from_utf8;

  #[derive(Deserialize)]
  struct LoginResponse {
    token: String,
    user_id: i32,
  }

  #[actix_rt::test]
  async fn test_user() {
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
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"CreateUser\":{\"username\":\"test_user\"}}}"
    );

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
        "query": "mutation UpdateUser($id: Int!, $password: String!) {
          UpdateUser(UpdateUser: { id: $id, password: $password }) {
            username
          }
        }",
        "variables": {
          "id": &login_response.user_id,
          "password": "changed",
        },
      }))
      .header("Authorization", login_response.token)
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Failed to update user password");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"UpdateUser\":{\"username\":\"test_user\"}}}"
    );

    let req = TestRequest::post()
      .uri("/login")
      .set_json(&json!({
        "username": "test_user",
        "password": "test",
      }))
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(
      resp.status().is_client_error(),
      "Login with old password still works"
    );

    let req = TestRequest::post()
      .uri("/login")
      .set_json(&json!({
        "username": "test_user",
        "password": "changed",
      }))
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "Login with new password failed");

    let body = test::read_body(resp).await;
    let body = from_utf8(&body).unwrap();
    let login_response: LoginResponse = serde_json::from_str(&body).unwrap();

    let req = TestRequest::post()
      .uri("/graphql")
      .set_json(&json!({
        "query": "mutation DeleteUser($id: Int!) {
          DeleteUser(DeleteUser: { id: $id }) {
            count
          }
        }",
        "variables": {
          "id": &login_response.user_id,
        },
      }))
      .header("Authorization", login_response.token)
      .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert!(resp.status().is_success(), "User deletion failed");
    assert_eq!(
      from_utf8(&test::read_body(resp).await).unwrap(),
      "{\"data\":{\"DeleteUser\":{\"count\":1}}}"
    );
  }
}
