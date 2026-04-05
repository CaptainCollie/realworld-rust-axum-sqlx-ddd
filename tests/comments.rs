use axum_test::expect_json;
use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestArticle, TestComment, TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_create_comment_success() {
    let (server, _container) = setup_test_app().await;

    let user = TestUser::new(&server, "cmt").await;

    let article = TestArticle::new(
        &server,
        "Comment Article",
        "For comments",
        "Article body",
        vec![],
        &user.token,
    )
    .await;

    let response = server
        .post(&format!("/api/articles/{}/comments", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({
            "comment": {
                "body": "Test comment body"
            }
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    response.assert_json_contains(&json!({
        "comment": {
            "id": expect_json::integer(),
            "body": "Test comment body",
            "createdAt": expect_json::iso_date_time().utc(),
            "updatedAt": expect_json::iso_date_time().utc(),
            "author": {
                "username": user.username,
                "bio": user.bio,
                "image": user.image,
                "following": false
            }
        }
    }));
}

#[tokio::test]
async fn test_list_comments_auth_and_anon() {
    let (server, _container) = setup_test_app().await;

    let user = TestUser::new(&server, "cmt").await;
    let article = TestArticle::new(&server, "Title", "D", "B", vec![], &user.token).await;
    let comment = TestComment::new(&server, &article.slug, "Test comment body", &user.token).await;

    let auth_res = server
        .get(&format!("/api/articles/{}/comments", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    auth_res.assert_status_ok();
    auth_res.assert_json_contains(&json!({
        "comments": expect_json::array().len(1).contains(vec![
            expect_json::object().contains(json!({
                "id": comment.id,
                "body": "Test comment body",
                "createdAt": expect_json::iso_date_time().utc(),
                "author": expect_json::object().contains(json!({ "username": user.username }))
        }))])
    }));

    let anon_res = server
        .get(&format!("/api/articles/{}/comments", article.slug))
        .await;

    anon_res.assert_status_ok();

    anon_res.assert_json_contains(&json!({
        "comments": [
            {
                "id": comment.id,
                "body": "Test comment body",
                "author": expect_json::object().contains(json!({ "username": user.username, "following": false }))
            }
        ]
    }));
}

#[tokio::test]
async fn test_selective_comment_deletion() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "del_user").await;
    let article = TestArticle::new(&server, "Title", "D", "B", vec![], &user.token).await;

    let comment1 = TestComment::new(&server, &article.slug, "First comment", &user.token).await;
    let _comment2 = TestComment::new(&server, &article.slug, "Second comment", &user.token).await;

    let list_before = server
        .get(&format!("/api/articles/{}/comments", article.slug))
        .await;
    list_before.assert_status_ok();
    list_before.assert_json_contains(&json!({ "comments": expect_json::array().len(2) }));

    let delete_res = server
        .delete(&format!(
            "/api/articles/{}/comments/{}",
            article.slug, comment1.id
        ))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    delete_res.assert_status(StatusCode::NO_CONTENT);

    let list_after = server
        .get(&format!("/api/articles/{}/comments", article.slug))
        .await;
    list_after.assert_status_ok();

    list_after.assert_json_contains(&json!({
        "comments": [
            { "body": "Second comment" }
        ]
    }));

    let body: serde_json::Value = list_after.json();
    assert_eq!(body["comments"].as_array().unwrap().len(), 1);

    assert_ne!(
        body["comments"][0]["id"].as_i64().unwrap(),
        comment1.id as i64
    );
}
