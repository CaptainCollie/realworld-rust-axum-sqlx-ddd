use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_profile_errors_specification() {
    let (server, _container) = setup_test_app().await;
    let unknown_name = "unknown-user-123";

    let response = server.get(&format!("/api/profiles/{}", unknown_name)).await;
    response.assert_status(StatusCode::NOT_FOUND);
    response.assert_json(&json!({
        "errors": { "profile": ["not found"] }
    }));

    let response = server
        .post(&format!("/api/profiles/{}/follow", unknown_name))
        .await;
    response.assert_status(StatusCode::UNAUTHORIZED);
    response.assert_json(&json!({
        "errors": { "token": ["is missing"] }
    }));

    let response = server
        .delete(&format!("/api/profiles/{}/follow", unknown_name))
        .await;
    response.assert_status(StatusCode::UNAUTHORIZED);
    response.assert_json(&json!({
        "errors": { "token": ["is missing"] }
    }));
}

#[tokio::test]
async fn test_profile_authed_not_found_errors() {
    let (server, _container) = setup_test_app().await;

    let main_user = TestUser::new(&server, "ep").await;
    let unknown_name = "unknown-user-999";

    let response = server
        .post(&format!("/api/profiles/{}/follow", unknown_name))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
    response.assert_json(&json!({
        "errors": { "profile": ["not found"] }
    }));

    let response = server
        .delete(&format!("/api/profiles/{}/follow", unknown_name))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
    response.assert_json(&json!({
        "errors": { "profile": ["not found"] }
    }));
}

#[tokio::test]
async fn test_error_follow_self_conflict() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "narcissus").await;

    let response = server
        .post(&format!("/api/profiles/{}/follow", user.username))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status(StatusCode::CONFLICT);

    response.assert_json_contains(&json!({
        "errors": {
            "body": ["You cannot follow yourself"]
        }
    }));
}
