use axum_test::expect_json;
use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestArticle, TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_article_feed_full_flow() {
    let (server, _container) = setup_test_app().await;

    let main_user = TestUser::new(&server, "feed_main").await;
    let celeb_user = TestUser::new(&server, "feed_celeb").await;

    let empty_feed = server
        .get("/api/articles/feed")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    empty_feed.assert_status_ok();
    empty_feed.assert_json_contains(&json!({
        "articles": [],
        "articlesCount": 0
    }));

    server
        .post(&format!("/api/profiles/{}/follow", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await
        .assert_status_ok();

    let art1 = TestArticle::new(&server, "Feed 1", "D1", "B1", vec![], &celeb_user.token).await;
    let _art2 = TestArticle::new(&server, "Feed 2", "D2", "B2", vec![], &celeb_user.token).await;

    let full_feed = server
        .get("/api/articles/feed")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    full_feed.assert_status_ok();
    full_feed.assert_json_contains(&json!({
        "articles": expect_json::array().len(2),
        "articlesCount": 2
    }));

    let limit_feed = server
        .get("/api/articles/feed?limit=1")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await;

    limit_feed.assert_status_ok();

    limit_feed.assert_json_contains(&json!({
        "articles": expect_json::array().len(1).all_contains(json!({
            "author": expect_json::object().contains(json!({
                    "username": celeb_user.username,
                    "following": true
            }))
        })),
        "articlesCount": 2
    }));

    server
        .delete(&format!("/api/articles/{}", art1.slug))
        .add_header("Authorization", format!("Token {}", celeb_user.token))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    server
        .delete(&format!("/api/profiles/{}/follow", celeb_user.username))
        .add_header("Authorization", format!("Token {}", main_user.token))
        .await
        .assert_status_ok();
}
