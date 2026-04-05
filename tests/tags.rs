use axum_test::expect_json;
use reqwest::StatusCode;
use serde_json::json;

use crate::common::{TestArticle, TestUser, setup_test_app};

mod common;

#[tokio::test]
async fn test_get_tags_list() {
    let (server, _container) = setup_test_app().await;
    let user = TestUser::new(&server, "tag_user").await;

    TestArticle::new(
        &server,
        "T",
        "D",
        "B",
        vec!["rust".into(), "axum".into()],
        &user.token,
    )
    .await;

    let response = server.get("/api/tags").await;

    response.assert_status(StatusCode::OK);
    response.assert_json_contains(&json!({
        "tags": expect_json::array()
            .min_len(2)
            .contains(["rust", "axum"])
    }));
}
