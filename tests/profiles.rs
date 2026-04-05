use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_get_profile_without_auth() {
    let (server, _container) = setup_test_app().await;

    let _main_user = TestUser::new(&server, "prof").await;

    let celeb_user = TestUser::new(&server, "celeb").await;

    let response = server
        .get(&format!("/api/profiles/{}", celeb_user.username))
        .await;
    response.assert_status(StatusCode::OK);

    response.assert_json(&json!({
        "profile": {
            "username": celeb_user.username,
            "bio": null,
            "image": null,
            "following": false
        }
    }));
}

#[tokio::test]
async fn test_get_profile_with_auth() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let celeb_user = TestUser::new(&server, "celeb").await;

    let response = server
        .get(&format!("/api/profiles/{}", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status(StatusCode::OK);

    response.assert_json(&json!({
        "profile": {
            "username": celeb_user.username,
            "bio": null,
            "image": null,
            "following": false
        }
    }));
}

#[tokio::test]
async fn test_profile_follow_flow() {
    let (server, _container) = setup_test_app().await;

    let main_user = TestUser::new(&server, "prof").await;
    let celeb_user = TestUser::new(&server, "celeb").await;

    let response = server
        .post(&format!("/api/profiles/{}/follow", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    response.assert_json(&json!({
        "profile": {
            "username": celeb_user.username,
            "bio": null,
            "image": null,
            "following": true
        }
    }));

    let anon_res = server
        .get(&format!("/api/profiles/{}", celeb_user.username))
        .await;

    anon_res.assert_json_contains(&serde_json::json!({
        "profile": {
            "following": false
        }
    }));

    let response = server
        .delete(&format!("/api/profiles/{}/follow", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    response.assert_json(&json!({
        "profile": {
            "username": celeb_user.username,
            "bio": null,
            "image": null,
            "following": false
        }
    }));

    let final_check = server
        .get(&format!("/api/profiles/{}", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    final_check.assert_json_contains(&serde_json::json!({
        "profile": {
            "following": false
        }
    }));

    let response = server
        .get(&format!("/api/profiles/{}", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    response.assert_status_ok();

    response.assert_json(&json!({
        "profile": {
            "username": celeb_user.username,
            "bio": null,
            "image": null,
            "following": false
        }
    }));
}
