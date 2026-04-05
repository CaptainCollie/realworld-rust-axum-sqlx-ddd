use axum::http::StatusCode;
use axum_test::expect_json;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::common::{TestArticle, TestProfile, TestUser, setup_test_app};
mod common;

#[tokio::test]
async fn test_create_article_with_tags_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "art").await;

    let title = format!("Test Article {}", user.username);
    let tags = vec!["tag1".to_string(), "tag2".to_string()];

    let response = server
        .post("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": {
                "title": title,
                "description": "Test description",
                "body": "Test body content",
                "tagList": tags
            }
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let body: Value = response.json();
    tracing::info!("{body}");
    let article_data = &body["article"];
    let article = TestArticle {
        slug: article_data["slug"].as_str().unwrap().to_string(),
        title: article_data["title"].as_str().unwrap().to_string(),
        description: article_data["description"].as_str().unwrap().to_string(),
        body: article_data["body"].as_str().unwrap().to_string(),
        tag_list: article_data["tagList"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect(),
        created_at: article_data["createdAt"]
            .as_str()
            .unwrap()
            .parse::<DateTime<Utc>>()
            .expect("Invalid createdAt format"),

        updated_at: article_data["updatedAt"]
            .as_str()
            .unwrap()
            .parse::<DateTime<Utc>>()
            .expect("Invalid updatedAt format"),
        favorited: article_data["favorited"].as_bool().unwrap_or(false),
        favorites_count: article_data["favoritesCount"].as_u64().unwrap_or(0) as u32,
        author: TestProfile {
            username: article_data["author"]["username"]
                .as_str()
                .unwrap()
                .to_string(),
            bio: article_data["author"]["bio"]
                .as_str()
                .map(|s| s.to_string()),
            image: article_data["author"]["image"]
                .as_str()
                .map(|s| s.to_string()),
            following: article_data["author"]["following"].as_bool().unwrap(),
        },
    };

    assert_eq!(article.title, title);
    assert!(!article.slug.is_empty());
    assert_eq!(article.description, "Test description");
    assert_eq!(article.body, "Test body content");

    assert_eq!(article.tag_list.len(), 2);
    assert!(article.tag_list.contains(&tags[0]));
    assert!(article.tag_list.contains(&tags[1]));

    assert!(!article.favorited);
    assert_eq!(article.favorites_count, 0);
    assert_eq!(article.author.username, user.username);

    assert!(article.created_at <= chrono::Utc::now());
}

#[tokio::test]
async fn test_list_articles_specification() {
    let (server, _container) = setup_test_app().await;

    let author = TestUser::new(&server, "lister").await;
    let title = "List Test Article";
    let description = "Description";

    TestArticle::new(
        &server,
        title,
        description,
        "Full Body Content",
        vec!["rust".into()],
        &author.token,
    )
    .await;

    let response = server.get("/api/articles").await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "articles": expect_json::array().min_len(1).all_contains(json!({
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "favorited": false,
            "favoritesCount": expect_json::integer(),

        })),
        "articlesCount": expect_json::integer().greater_than(0),
    }));

    let body: Value = response.json();
    let first_article = &body["articles"][0];

    assert_eq!(first_article["title"], title);
    assert_eq!(first_article["description"], description);

    assert!(
        first_article["body"].is_null() || first_article.get("body").is_none(),
        "Body should not be present in list view"
    );

    assert_eq!(first_article["author"]["username"], author.username);
}

#[tokio::test]
async fn test_list_by_author_specification() {
    let (server, _container) = setup_test_app().await;

    let author = TestUser::new(&server, "art").await;
    let title = "Author Filter Test";
    let description = "Description";

    TestArticle::new(
        &server,
        title,
        description,
        "body",
        vec!["rust".into()],
        &author.token,
    )
    .await;

    let response = server
        .get(&format!("/api/articles?author={}", author.username))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "articles": expect_json::array().min_len(1).all_contains(json!({
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "favorited": false,
            "favoritesCount": expect_json::integer(),
            "author": expect_json::object().contains(json!({
                "username": author.username
            }))
        })),
        "articlesCount": expect_json::integer().greater_than(0),
    }));

    let body: Value = response.json();
    let first_article = &body["articles"][0];

    assert_eq!(first_article["title"].as_str().unwrap(), title);
    assert_eq!(first_article["description"].as_str().unwrap(), description);

    assert!(first_article.get("body").is_none());
}

#[tokio::test]
async fn test_list_articles_with_auth_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "authed").await;

    TestArticle::new(
        &server,
        "Auth Test",
        "...",
        "...",
        vec!["rust".into()],
        &user.token,
    )
    .await;

    let response = server
        .get("/api/articles")
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "articles": expect_json::array().min_len(1).all_contains(json!({
            "title": expect_json::string(),
            "slug": expect_json::string(),
            "description": expect_json::string(),
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time(),
            "updatedAt": expect_json::iso_date_time(),
            "favorited": false,
            "favoritesCount": expect_json::integer(),
            "author": expect_json::object().contains(json!({
                "username": user.username,
            }))
        })),
        "articlesCount": expect_json::integer().greater_than(0),
    }));

    let body: Value = response.json();
    let first_article = &body["articles"][0];

    assert!(first_article.get("body").is_none());
}

#[tokio::test]
async fn test_list_by_author_with_auth_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "authed").await;

    TestArticle::new(
        &server,
        "Auth Test",
        "...",
        "...",
        vec!["rust".into()],
        &user.token,
    )
    .await;

    let response = server
        .get(&format!("/api/articles?author={}", user.username))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "articles": expect_json::array().min_len(1).all_contains(json!({
            "title": expect_json::string(),
            "slug": expect_json::string(),
            "description": expect_json::string(),
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time(),
            "updatedAt": expect_json::iso_date_time(),
            "favorited": false,
            "favoritesCount": expect_json::integer(),
            "author": expect_json::object().contains(json!({
                "username": user.username,
            }))
        })),
        "articlesCount": expect_json::integer().greater_than(0),
    }));

    let body: Value = response.json();
    let first_article = &body["articles"][0];

    assert!(first_article.get("body").is_none());
}

#[tokio::test]
async fn test_list_by_tag_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "tagger").await;

    let target_tag = format!("tag_{}", user.username);
    TestArticle::new(
        &server,
        "Tag Test Article",
        "desc",
        "body",
        vec![target_tag.clone(), "other_tag".into()],
        &user.token,
    )
    .await;

    let response = server
        .get(&format!("/api/articles?tag={}", target_tag))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "articles": expect_json::array().min_len(1).all_contains(json!({
            "title": expect_json::string(),
            "slug": expect_json::string(),
            "description": expect_json::string(),
            "tagList": expect_json::array().contains([target_tag]),
            "createdAt": expect_json::iso_date_time(),
            "favorited": false,
            "favoritesCount": expect_json::integer(),
            "author": expect_json::object().contains(json!({
                "username": user.username,
            }))
        })),
        "articlesCount": expect_json::integer().greater_than(0),
    }));

    let body: Value = response.json();
    let first_article = &body["articles"][0];

    assert!(first_article.get("body").is_none());
}

#[tokio::test]
async fn test_get_single_article_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "art").await;

    let title = "Test Article";
    let article = TestArticle::new(
        &server,
        title,
        "Test description",
        "Test body content",
        vec!["rust".into()],
        &user.token,
    )
    .await;

    let response = server.get(&format!("/api/articles/{}", article.slug)).await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "article": {
            "title": title,
            "slug": article.slug,
            "description": "Test description",
            "body": "Test body content",
            "tagList": expect_json::array().contains(["rust"]),
            "createdAt": expect_json::iso_date_time(),
            "updatedAt": expect_json::iso_date_time(),
            "favorited": false,
            "favoritesCount": 0,
            "author": {
                "username": user.username
            }
        }
    }));
}

#[tokio::test]
async fn test_update_article_body_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "art").await;

    let article = TestArticle::new(
        &server,
        "Test Article",
        "desc",
        "old body",
        vec!["tag1".into(), "tag2".into()],
        &user.token,
    )
    .await;

    let response = server
        .put(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "article": { "body": "Updated body content" }
        }))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "article": {
            "title": article.title,
            "body": "Updated body content",
            "tagList": expect_json::array().min_len(1),
        }
    }));

    let body: serde_json::Value = response.json();
    assert_ne!(body["article"]["updatedAt"], body["article"]["createdAt"]);

    let response = server
        .get(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "article": {
            "title": article.title,
            "slug": article.slug,
            "description": article.description,
            "body": "Updated body content",
            "tagList": [
                &article.tag_list[0],
                &article.tag_list[1]
            ],
            "createdAt": expect_json::iso_date_time(),
            "updatedAt": expect_json::iso_date_time(),
            "author": {
                "username": user.username
            }
        }
    }));

    let final_body: serde_json::Value = response.json();
    assert_ne!(
        final_body["article"]["updatedAt"], final_body["article"]["createdAt"],
        "updatedAt must be different from createdAt after update"
    );
}

#[tokio::test]
async fn test_article_tags_update_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "tagmaster").await;

    let article = TestArticle::new(
        &server,
        "Title",
        "Desc",
        "Body",
        vec!["tag1".into(), "tag2".into()],
        &user.token,
    )
    .await;

    server
        .put(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({ "article": { "body": "New Body" } }))
        .await
        .assert_json_contains(&json!({
            "article": {
                "tagList": expect_json::array().len(2)
            }
        }));

    server
        .put(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({ "article": { "tagList": [] } }))
        .await
        .assert_json_contains(&json!({
            "article": {
                "tagList": expect_json::array().len(0)
            }
        }));

    server
        .put(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({ "article": { "tagList": null } }))
        .await
        .assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_delete_article_specification() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "deleter").await;

    let article = TestArticle::new(
        &server,
        "To Be Deleted",
        "desc",
        "body",
        vec![],
        &user.token,
    )
    .await;

    let response = server
        .delete(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status(StatusCode::NO_CONTENT);

    let verify_res = server.get(&format!("/api/articles/{}", article.slug)).await;

    verify_res.assert_status(StatusCode::NOT_FOUND);

    verify_res.assert_json_contains(&json!({
        "errors": {
            "article": ["not found"]
        }
    }));
}
