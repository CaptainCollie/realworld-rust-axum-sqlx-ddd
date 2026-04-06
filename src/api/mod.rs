pub mod dto;
pub mod extractor;
pub mod handlers;

use crate::{
    api::{
        extractor::AuthConfig,
        handlers::{
            articles::{
                create_article, delete_article, favorite_article, get_article, get_feed, get_tags,
                list_articles, unfavorite_article, update_article,
            },
            comments::{add_comment, delete_comment, get_comments},
            profile::{follow_user, get_profile, unfollow_user},
            users::{get_current_user, login, register, update_current_user},
        },
    },
    application::services::{
        article_service::ArticleService, comment_service::CommentService,
        profile_service::ProfileService, user_service::UserService,
    },
};
use axum::{
    Router,
    body::Body,
    extract::MatchedPath,
    http::Request,
    middleware::Next,
    response::IntoResponse,
    routing::{delete, get, post},
};
use metrics::{counter, histogram};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::TraceLayer,
};
use uuid::Uuid;

#[derive(Clone, Copy)]
struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().ok()?;
        Some(RequestId::new(request_id))
    }
}

pub fn create_router(
    user_service: Arc<UserService>,
    profile_service: Arc<ProfileService>,
    article_service: Arc<ArticleService>,
    comment_service: Arc<CommentService>,
    recorder_handle: PrometheusHandle,
) -> Router {
    let metrics_router = Router::new().route(
        "/metrics",
        get(move || std::future::ready(recorder_handle.render())),
    );

    Router::new()
        .merge(metrics_router)
        .route("/api/users", post(register))
        .route("/api/users/login", post(login))
        .route("/api/user", get(get_current_user).put(update_current_user))
        .route("/api/profiles/{username}", get(get_profile))
        .route("/api/articles", get(list_articles).post(create_article))
        .route("/api/articles/feed", get(get_feed))
        .route(
            "/api/articles/{slug}",
            get(get_article).delete(delete_article).put(update_article),
        )
        .route(
            "/api/articles/{slug}/favorite",
            post(favorite_article).delete(unfavorite_article),
        )
        .route(
            "/api/profiles/{username}/follow",
            post(follow_user).delete(unfollow_user),
        )
        .route("/api/tags", get(get_tags))
        .route(
            "/api/articles/{slug}/comments",
            get(get_comments).post(add_comment),
        )
        .route("/api/articles/{slug}/comments/{id}", delete(delete_comment))
        .with_state(AppState {
            user_service,
            profile_service,
            article_service,
            comment_service,
        })
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MyMakeRequestId)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &Request<_>| {
                            let request_id = request
                                .extensions()
                                .get::<RequestId>()
                                .map(|ri| ri.header_value().to_str().unwrap_or_default())
                                .unwrap_or_default();

                            tracing::info_span!(
                                "http_request",
                                method = %request.method(),
                                uri = %request.uri(),
                                request_id = %request_id,
                            )
                        })
                        .on_response(
                            tower_http::trace::DefaultOnResponse::new()
                                .level(tracing::Level::INFO)
                                .latency_unit(tower_http::LatencyUnit::Millis),
                        ),
                )
                .layer(axum::middleware::from_fn(track_metrics))
                .propagate_x_request_id(),
        )
}

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub profile_service: Arc<ProfileService>,
    pub article_service: Arc<ArticleService>,
    pub comment_service: Arc<CommentService>,
}

impl AuthConfig for AppState {
    fn jwt_secret(&self) -> &str {
        self.user_service.jwt_secret()
    }
}

async fn track_metrics(req: Request<Body>, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        "unknown_route".to_owned()
    };
    let method = req.method().to_string();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    counter!("http_requests_total", "method" => method.clone(), "path" => path.clone(), "status" => status.clone())
        .increment(1);
    histogram!("http_request_duration_seconds", "method" => method.clone(), "path" => path.clone())
        .record(latency);

    response
}
