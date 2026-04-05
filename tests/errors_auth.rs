mod common;

use reqwest::StatusCode;

use crate::common::{TestUser, setup_test_app};

#[tokio::test]
async fn test_register_empty_username_error() {
    let (server, _container) = setup_test_app().await;

    let response = server
        .post("/api/users")
        .json(&serde_json::json!({
            "user": {
                "username": "",
                "email": "ea_blank_123@test.com",
                "password": "password123"
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    response.assert_json(&serde_json::json!({
        "errors": {
            "username": ["can't be blank"]
        }
    }));
}

#[tokio::test]
async fn test_register_empty_email_error() {
    let (server, _container) = setup_test_app().await;
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();

    let response = server
        .post("/api/users")
        .json(&serde_json::json!({
            "user": {
                "username": format!("ea_blank_{}", uid),
                "email": "",
                "password": "password123"
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    response.assert_json(&serde_json::json!({
        "errors": {
            "email": ["can't be blank"]
        }
    }));
}

#[tokio::test]
async fn test_register_empty_password_error() {
    let (server, _container) = setup_test_app().await;
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();

    let response = server
        .post("/api/users")
        .json(&serde_json::json!({
            "user": {
                "username": format!("ea_blank_{}", uid),
                "email": format!("ea_blankp_{}@test.com", uid),
                "password": ""
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    response.assert_json(&serde_json::json!({
        "errors": {
            "password": ["can't be blank"]
        }
    }));
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .post("/api/users")
        .json(&serde_json::json!({
            "user": {
                "username": main_user.username,
                "email": "other@test.com",
                "password": "password123"
            }
        }))
        .await;

    response.assert_status(StatusCode::CONFLICT);

    response.assert_json(&serde_json::json!({
        "errors": {
            "username": ["has already been taken"]
        }
    }));
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .post("/api/users")
        .json(&serde_json::json!({
            "user": {
                "username": "otherusername",
                "email": main_user.email,
                "password": "password123"
            }
        }))
        .await;

    response.assert_status(StatusCode::CONFLICT);

    response.assert_json(&serde_json::json!({
        "errors": {
            "email": ["has already been taken"]
        }
    }));
}

#[tokio::test]
async fn test_login_empty_email_error() {
    let (server, _container) = setup_test_app().await;

    let response = server
        .post("/api/users/login")
        .json(&serde_json::json!({
            "user": {
                "email": "",
                "password": "password123"
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    response.assert_json(&serde_json::json!({
        "errors": {
            "email": ["can't be blank"]
        }
    }));
}

#[tokio::test]
async fn test_login_empty_password_error() {
    let (server, _container) = setup_test_app().await;
    let uid = uuid::Uuid::new_v4().to_string()[..8].to_string();

    let response = server
        .post("/api/users/login")
        .json(&serde_json::json!({
            "user": {
                "email": format!("ea_dup_{}@test.com", uid),
                "password": ""
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

    response.assert_json(&serde_json::json!({
        "errors": {
            "password": ["can't be blank"]
        }
    }));
}

#[tokio::test]
async fn test_login_wrong_password_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .post("/api/users/login")
        .json(&serde_json::json!({
            "user": {
                "email": main_user.email,
                "password": "wrongpassword"
            }
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    response.assert_json(&serde_json::json!({
        "errors": {
            "credentials": ["invalid"]
        }
    }));
}

#[tokio::test]
async fn test_login_without_token_error() {
    let (server, _container) = setup_test_app().await;

    let response = server.get("/api/user").await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    response.assert_json(&serde_json::json!({
        "errors": {
            "token": ["is missing"]
        }
    }));

    let response = server
        .put("/api/user")
        .json(&serde_json::json!({
                    "user": {
                        "bio": "test"
                    }
        }))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);

    response.assert_json(&serde_json::json!({
        "errors": {
            "token": ["is missing"]
        }
    }));
}

#[tokio::test]
async fn test_update_email_to_empty_string_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&serde_json::json!({
            "user": {
                "email": ""
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_update_username_to_empty_string_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&serde_json::json!({
            "user": {
                "username": ""
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_update_password_to_empty_string_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&serde_json::json!({
            "user": {
                "password": ""
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_update_email_to_null_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&serde_json::json!({
            "user": {
                "email": None::<String>
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_update_username_to_null_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&serde_json::json!({
            "user": {
                "username": None::<String>
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_update_password_to_null_error() {
    let (server, _container) = setup_test_app().await;
    let main_user = TestUser::new(&server, "prof").await;

    let response = server
        .put("/api/user")
        .add_header("Authorization", format!("Token {}", main_user.token))
        .json(&serde_json::json!({
            "user": {
                "password": None::<String>
            }
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}
