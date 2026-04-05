use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestArticle, TestUser, setup_test_app};

mod common;


#[tokio::test]
async fn test_comment_actions_no_auth() {
    let (server, _container) = setup_test_app().await;

    let post_res = server
        .post("/api/articles/any-slug/comments")
        .json(&json!({ "comment": { "body": "test" } }))
        .await;

    post_res.assert_status(StatusCode::UNAUTHORIZED);
    post_res.assert_json_contains(&json!({
        "errors": { "token": ["is missing"] }
    }));

    let delete_res = server
        .delete("/api/articles/any-slug/comments/1")
        .await;

    delete_res.assert_status(StatusCode::UNAUTHORIZED);
    delete_res.assert_json_contains(&json!({
        "errors": { "token": ["is missing"] }
    }));
}

#[tokio::test]
async fn test_comment_validation_and_not_found() {
    let (server, _container) = setup_test_app().await;
    
    let user = TestUser::new(&server, "ec").await;
    let article = TestArticle::new(&server, "Err Art", "D", "B", vec![], &user.token).await;

    let empty_res = server
        .post(&format!("/api/articles/{}/comments", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({ "comment": { "body": "" } }))
        .await;

    empty_res.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    empty_res.assert_json_contains(&json!({
        "errors": { "body": ["can't be blank"] }
    }));

    let unknown_slug = format!("unknown-slug-{}", uuid::Uuid::new_v4());
    let unknown_res = server
        .post(&format!("/api/articles/{}/comments", unknown_slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .json(&json!({ "comment": { "body": "orphan comment" } }))
        .await;

    unknown_res.assert_status(StatusCode::NOT_FOUND);
    unknown_res.assert_json_contains(&json!({
        "errors": { "article": ["not found"] }
    }));
}

#[tokio::test]
async fn test_get_comments_on_unknown_article() {
    let (server, _container) = setup_test_app().await;

    let unknown_slug = format!("unknown-slug-{}", uuid::Uuid::new_v4());

    let response = server
        .get(&format!("/api/articles/{}/comments", unknown_slug))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);

    response.assert_json_contains(&json!({
        "errors": {
            "article": ["not found"]
        }
    }));
}

#[tokio::test]
async fn test_delete_comment_not_found_scenarios() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "err_del").await;
    let article = TestArticle::new(&server, "Exist Art", "D", "B", vec![], &user.token).await;

    let unknown_slug = format!("unknown-slug-{}", uuid::Uuid::new_v4());
    let res_art = server
        .delete(&format!("/api/articles/{}/comments/99999", unknown_slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    res_art.assert_status(StatusCode::NOT_FOUND);
    res_art.assert_json_contains(&json!({
        "errors": { "article": ["not found"] }
    }));

    let res_cmt = server
        .delete(&format!("/api/articles/{}/comments/99999", article.slug))
        .add_header("Authorization", format!("Token {}", user.token))
        .await;

    res_cmt.assert_status(StatusCode::NOT_FOUND);
    res_cmt.assert_json_contains(&json!({
        "errors": { "comment": ["not found"] }
    }));
}
