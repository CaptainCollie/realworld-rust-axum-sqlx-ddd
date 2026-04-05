use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestArticle, TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_article_authorization_security() {
    let (server, _container) = setup_test_app().await;

    let user_a = TestUser::new(&server, "user_a").await;
    let user_b = TestUser::new(&server, "user_b").await;

    let article = TestArticle::new(&server, "Private Title", "D", "B", vec![], &user_a.token).await;

    let delete_res = server
        .delete(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user_b.token))
        .await;

    delete_res.assert_status(StatusCode::FORBIDDEN);
    delete_res.assert_json_contains(&json!({
        "errors": { "article": ["forbidden"] }
    }));

    let update_res = server
        .put(&format!("/api/articles/{}", article.slug))
        .add_header("Authorization", format!("Token {}", user_b.token))
        .json(&json!({"article": {"body": "hacked"}}))
        .await;

    update_res.assert_status(StatusCode::FORBIDDEN);
    update_res.assert_json_contains(&json!({
        "errors": { "article": ["forbidden"] }
    }));
}
