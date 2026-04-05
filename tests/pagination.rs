use serde_json::json;

use crate::common::{TestArticle, TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_articles_pagination_and_sorting() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "page_user").await;

    let art1 = TestArticle::new(
        &server,
        "Pagination 1",
        "Desc 1",
        "Body 1",
        vec![],
        &user.token,
    )
    .await;

    let art2 = TestArticle::new(
        &server,
        "Pagination 2",
        "Desc 2",
        "Body 2",
        vec![],
        &user.token,
    )
    .await;

    let page1_res = server
        .get(&format!("/api/articles?author={}&limit=1", user.username))
        .await;

    page1_res.assert_status_ok();
    page1_res.assert_json_contains(&json!({
        "articles": [ { "slug": art2.slug } ],
        "articlesCount": 2
    }));

    let body1: serde_json::Value = page1_res.json();
    assert_eq!(body1["articles"].as_array().unwrap().len(), 1);

    let page2_res = server
        .get(&format!(
            "/api/articles?author={}&limit=1&offset=1",
            user.username
        ))
        .await;

    page2_res.assert_status_ok();
    page2_res.assert_json_contains(&json!({
        "articles": [ { "slug": art1.slug } ],
        "articlesCount": 2
    }));

    let body2: serde_json::Value = page2_res.json();
    assert_eq!(body2["articles"].as_array().unwrap().len(), 1);
}
