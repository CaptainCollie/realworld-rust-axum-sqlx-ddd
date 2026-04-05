use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_create_article_no_auth() {
    let (server, _container) = setup_test_app().await;

    let response = server
        .post("/api/articles")
        .json(&json!({
            "article": {
                "title": "No Auth Article",
                "description": "test",
                "body": "test"
            }
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    response.assert_json_contains(&json!({
        "errors": {
            "token": ["is missing"]
        }
    }));
}

#[tokio::test]
async fn test_delete_article_no_auth() {
    let (server, _container) = setup_test_app().await;

    let response = server.delete("/api/articles/some-slug").await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    response.assert_json_contains(&json!({
        "errors": {
            "token": ["is missing"]
        }
    }));
}

#[tokio::test]
async fn test_get_feed_no_auth() {
    let (server, _container) = setup_test_app().await;

    let response = server.get("/api/articles/feed").await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    response.assert_json_contains(&json!({
        "errors": {
            "token": ["is missing"]
        }
    }));
}

#[tokio::test]
async fn test_create_article_validation_errors() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "val_tester").await;

    let res_title = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": { "title": "", "description": "test", "body": "test" }
        }))
        .await;

    res_title.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    res_title.assert_json_contains(&json!({
        "errors": { "title": ["can't be blank"] }
    }));

    let res_desc = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": { "title": "Some Title", "description": "", "body": "test" }
        }))
        .await;

    res_desc.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    res_desc.assert_json_contains(&json!({
        "errors": { "description": ["can't be blank"] }
    }));

    let res_body = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": { "title": "Some Title", "description": "test", "body": "" }
        }))
        .await;

    res_body.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    res_body.assert_json_contains(&json!({
        "errors": { "body": ["can't be blank"] }
    }));
}

#[tokio::test]
async fn test_duplicate_titles_get_unique_slugs() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "dup_tester").await;

    let title = "Duplicate Title";

    let res1 = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": {
                "title": title,
                "description": "first article",
                "body": "first body"
            }
        }))
        .await;

    res1.assert_status(StatusCode::CREATED);
    let body1: serde_json::Value = res1.json();
    let slug1 = body1["article"]["slug"].as_str().unwrap();

    let res2 = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": {
                "title": title,
                "description": "second article",
                "body": "second body"
            }
        }))
        .await;

    res2.assert_status(StatusCode::CREATED);
    let body2: serde_json::Value = res2.json();
    let slug2 = body2["article"]["slug"].as_str().unwrap();

    assert_ne!(
        slug1, slug2,
        "Каждый слаг должен быть уникальным, даже при одинаковых заголовках"
    );

    assert!(!slug1.is_empty());
    assert!(!slug2.is_empty());
}

#[tokio::test]
async fn test_article_actions_on_unknown_slug() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "not_found_tester").await;
    let auth_header = format!("Token {}", user.token);

    let slug = format!("unknown-slug-{}", uuid::Uuid::new_v4());

    let endpoints = vec![
        ("GET", format!("/api/articles/{}", slug), None),
        (
            "PUT",
            format!("/api/articles/{}", slug),
            Some(json!({"article": {"body": "test"}})),
        ),
        ("POST", format!("/api/articles/{}/favorite", slug), None),
        ("DELETE", format!("/api/articles/{}/favorite", slug), None),
        ("DELETE", format!("/api/articles/{}", slug), None),
    ];

    for (method, path, body) in endpoints {
        let mut request = match method {
            "GET" => server.get(&path),
            "PUT" => server.put(&path),
            "POST" => server.post(&path),
            "DELETE" => server.delete(&path),
            _ => unreachable!(),
        };

        request = request.add_header("Authorization", &auth_header);

        if let Some(b) = body {
            request = request.json(&b);
        }

        let response = request.await;

        response.assert_status(StatusCode::NOT_FOUND);

        response.assert_json_contains(&json!({
            "errors": {
                "article": ["not found"]
            }
        }));
    }
}
