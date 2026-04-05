mod common;

use crate::common::{TestUser, setup_test_app};
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn test_register_specification() {
    let (server, _container) = setup_test_app().await;

    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let username = format!("auth_{}", uid);
    let email = format!("auth_{}@test.com", uid);
    let password = "password123";

    let response = server
        .post("/api/users")
        .json(&json!({
            "user": {
                "username": &username,
                "email": &email,
                "password": password
            }
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: Value = response.json();

    assert_eq!(body["user"]["username"], username);
    assert_eq!(body["user"]["email"], email);
    assert!(
        body["user"]["bio"].is_null(),
        "Bio should be null on registration"
    );
    assert!(
        body["user"]["image"].is_null(),
        "Image should be null on registration"
    );
    assert!(
        body["user"]["token"].is_string(),
        "Token should be a string"
    );
    let token = body["user"]["token"].as_str().unwrap();
    assert!(!token.is_empty(), "Token should not be empty");

    println!("Captured reg_token: {}", token);
}

#[tokio::test]
async fn test_login_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .post("/api/users/login")
        .json(&json!({
            "user": {
                "email": main_user.email,
                "password": "password123"
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["username"], main_user.username);
    assert_eq!(body["user"]["email"], main_user.email);
    assert!(body["user"]["bio"].is_null());
    assert!(body["user"]["image"].is_null());

    let token = body["user"]["token"]
        .as_str()
        .expect("Token should be a string");
    assert!(!token.is_empty());

    println!("Captured token: {}", token);
}

#[tokio::test]
async fn test_get_current_user_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["username"], main_user.username);
    assert_eq!(body["user"]["email"], main_user.email);
    assert!(body["user"]["bio"].is_null());
    assert!(body["user"]["image"].is_null());
    assert!(body["user"]["token"].is_string());
    assert!(!body["user"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_update_user_bio_specification() {
    let (server, _container) = common::setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": "Updated bio"
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["username"], main_user.username);
    assert_eq!(body["user"]["email"], main_user.email);
    assert_eq!(body["user"]["bio"], "Updated bio");
    assert!(body["user"]["image"].is_null());
    assert!(body["user"]["token"].is_string());
    assert!(!body["user"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_verify_update_persisted_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": "Updated bio"
            }
        }))
        .await
        .assert_status_ok();

    let response = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["username"], main_user.username);
    assert_eq!(body["user"]["email"], main_user.email);
    assert_eq!(body["user"]["bio"], "Updated bio");
    assert!(body["user"]["image"].is_null());
    assert!(body["user"]["token"].is_string());
    assert!(!body["user"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_update_user_bio_empty_normalization_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": ""
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert!(body["user"]["bio"].is_null());
}

#[tokio::test]
async fn test_verify_empty_string_normalization_persisted() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": ""
            }
        }))
        .await
        .assert_status_ok();

    let response = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert!(body["user"]["bio"].is_null());
}

#[tokio::test]
async fn test_restore_bio_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": "Temporary bio"
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["bio"], "Temporary bio");
}

#[tokio::test]
async fn test_update_user_bio_to_null_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": null
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert!(body["user"]["bio"].is_null());
}

#[tokio::test]
async fn test_verify_null_bio_persisted_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({ "user": { "bio": null } }))
        .await
        .assert_status_ok();

    let response = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert!(body["user"]["bio"].is_null());
}

#[tokio::test]
async fn test_restore_bio_persisted_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "bio": "Updated bio"
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["username"], main_user.username);
    assert_eq!(body["user"]["email"], main_user.email);
    assert_eq!(body["user"]["bio"], "Updated bio");
    assert!(body["user"]["image"].is_null());
    assert!(body["user"]["token"].is_string());
    assert!(!body["user"]["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_update_user_image_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "image": "https://example.com/photo.jpg"
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["image"], "https://example.com/photo.jpg");
}

#[tokio::test]
async fn test_verify_image_update_persisted_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "image": "https://example.com/photo.jpg"
            }
        }))
        .await
        .assert_status_ok();

    let response = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["image"], "https://example.com/photo.jpg");
}

#[tokio::test]
async fn test_update_user_image_empty_normalization_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "image": ""
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert!(body["user"]["image"].is_null());
}

#[tokio::test]
async fn test_verify_image_empty_string_normalization_persisted() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "image": ""
            }
        }))
        .await
        .assert_status_ok();

    let response = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert!(body["user"]["image"].is_null());
}

#[tokio::test]
async fn test_update_user_image_to_null_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "image": "https://example.com/temp.jpg"
            }
        }))
        .await
        .assert_status_ok();

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "image": null
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();
    assert!(body["user"]["image"].is_null());
}

#[tokio::test]
async fn test_update_username_and_email_specification() {
    let (server, _container) = common::setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;

    server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({ "user": { "bio": "Updated bio" } }))
        .await
        .assert_status_ok();

    let res_get = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;
    res_get.assert_status_ok();
    assert!(res_get.json::<serde_json::Value>()["user"]["image"].is_null());

    let new_username = "new_username";
    let new_email = "new_email@test.com";

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "username": new_username,
                "email": new_email,
            }
        }))
        .await;

    response.assert_status_ok();

    let body = response.json::<serde_json::Value>();

    assert_eq!(body["user"]["username"], new_username);
    assert_eq!(body["user"]["email"], new_email);
    assert_eq!(body["user"]["bio"], "Updated bio");
    assert!(body["user"]["image"].is_null());

    let updated_token = body["user"]["token"]
        .as_str()
        .expect("Token should be a string");
    assert!(!updated_token.is_empty());
}

#[tokio::test]
async fn test_verify_username_email_update_persisted_with_new_token() {
    let (server, _container) = common::setup_test_app().await;
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    let main_user = TestUser::new(&server, "prof").await;

    let new_username = format!("auth_{}_upd", uid);
    let new_email = format!("auth_{}_upd@test.com", uid);

    let update_resp = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&json!({
            "user": {
                "username": &new_username,
                "email": &new_email
            }
        }))
        .await;

    update_resp.assert_status_ok();

    let body = update_resp.json::<serde_json::Value>();
    let updated_token = body["user"]["token"]
        .as_str()
        .expect("Token missing in update response")
        .to_string();

    let final_res = server
        .get("/api/user")
        .add_header("Authorization", format!("Token {}", updated_token))
        .await;

    final_res.assert_status_ok();
    let final_body = final_res.json::<serde_json::Value>();
    assert_eq!(final_body["user"]["username"], new_username);
    assert_eq!(final_body["user"]["email"], new_email);
}
