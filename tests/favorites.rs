use axum_test::expect_json;
use reqwest::StatusCode;
use serde_json::{Value, json};

use crate::common::{TestArticle, TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_article_favorite_assertions() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "fav_tester").await;
    let article = TestArticle::new(&server, "Title", "Desc", "Body", vec![], &user.token).await;

    let response = server
        .post(&format!("/api/articles/{}/favorite", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "article": {
            "slug": article.slug,
            "title": "Title",
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "favorited": true,
            "favoritesCount": 1,
            "author": {
                "username": user.username,
                "following": false
            }
        }
    }));

    let response = server
        .get(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    response.assert_status_ok();

    response.assert_json_contains(&json!({
        "article": {
            "slug": article.slug,
            "title": "Title",
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "favorited": true,
            "favoritesCount": 1,
            "author": {
                "username": user.username,
                "following": false
            }
        }
    }));
}

#[tokio::test]
async fn test_list_articles_filtered_by_favorited() {
    let (server, _container) = setup_test_app().await;

    let author = TestUser::new(&server, "author").await;
    let fan = TestUser::new(&server, "fan").await;

    let article = TestArticle::new(
        &server,
        "Filter Me",
        "Desc",
        "Body",
        vec!["rust".into()],
        &author.token,
    )
    .await;

    server
        .post(&format!("/api/articles/{}/favorite", article.slug))
        .add_header("Authorization", format!("Token {}", fan.token))
        .await
        .assert_status(StatusCode::OK);

    let anon_response = server
        .get(&format!("/api/articles?favorited={}", fan.username))
        .await;

    anon_response.assert_status_ok();
    anon_response.assert_json_contains(&json!({
        "articles": expect_json::array().contains(
            [expect_json::object().contains(json!({
            "slug": article.slug,
            "title": article.title,
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "favorited": false,
            "favoritesCount": 1,
        }))]),
        "articlesCount": 1
    }));

    let body: Value = anon_response.json();
    let first_article = &body["articles"][0];

    assert!(first_article.get("body").is_none());

    let auth_response = server
        .get(&format!("/api/articles?favorited={}", fan.username))
        .add_header("Authorization", format!("Token {}", fan.token))
        .await;

    auth_response.assert_status_ok();
    auth_response.assert_json_contains(&json!({
        "articles": expect_json::array().contains(
            [expect_json::object().contains(json!({
            "slug": article.slug,
            "title": article.title,
            "tagList": expect_json::array(),
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "favorited": true,
            "favoritesCount": 1,
        }))]),
        "articlesCount": 1
    }));

    let body: Value = auth_response.json();
    let first_article = &body["articles"][0];

    assert!(first_article.get("body").is_none());
}

#[tokio::test]
async fn test_unfavorite_article_lifecycle() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "unfav_user").await;

    let article =
        TestArticle::new(&server, "Unfav Title", "Desc", "Body", vec![], &user.token).await;

    server
        .post(&format!("/api/articles/{}/favorite", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await
        .assert_status_ok();

    let unfav_response = server
        .delete(&format!("/api/articles/{}/favorite", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    unfav_response.assert_status_ok();
    unfav_response.assert_json_contains(&json!({
        "article": {
            "slug": article.slug,
            "title": article.title,
            "body": article.body,
            "favorited": false,
            "favoritesCount": 0,
            "createdAt": expect_json::iso_date_time().utc(),
            "author": {
                "username": user.username
            }
        }
    }));

    let get_response = server
        .get(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    get_response.assert_status_ok();
    get_response.assert_json_contains(&json!({
        "article": {
            "favorited": false,
            "favoritesCount": 0
        }
    }));
}

#[tokio::test]
async fn test_delete_article_cleanup() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "cleanup_user").await;

    let article = TestArticle::new(
        &server,
        "Delete Me",
        "Desc",
        "Body",
        vec!["test".into()],
        &user.token,
    )
    .await;

    let delete_response = server
        .delete(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    delete_response.assert_status(StatusCode::NO_CONTENT);

    let get_response = server.get(&format!("/api/articles/{}", article.slug)).await;

    get_response.assert_status(StatusCode::NOT_FOUND);

    get_response.assert_json_contains(&json!({
        "errors": {
            "article": ["not found"]
        }
    }));
}
