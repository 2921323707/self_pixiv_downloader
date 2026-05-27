use std::net::SocketAddr;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, post, put};
use tower_http::cors::{Any, CorsLayer};

use super::AppState;
use super::handlers::downloads::{
    post_download_author, post_download_bookmarks, post_download_single,
};
use super::handlers::health::get_health;
use super::handlers::images::{
    delete_image, get_image, get_image_file, list_images, post_delete_images,
};
use super::handlers::settings::{get_settings, post_test_deepseek, post_test_pixiv, put_setting};
use super::handlers::smart::{post_smart_download, post_smart_parse};
use super::handlers::tasks::{get_task, list_tasks};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(get_health))
        .route("/api/download/single", post(post_download_single))
        .route("/api/downloads/single", post(post_download_single))
        .route("/api/downloads/bookmarks", post(post_download_bookmarks))
        .route("/api/downloads/author", post(post_download_author))
        .route("/api/smart/parse", post(post_smart_parse))
        .route("/api/smart/download", post(post_smart_download))
        .route("/api/images", get(list_images))
        .route("/api/images/delete-batch", post(post_delete_images))
        .route("/api/images/{image_id}/file", get(get_image_file))
        .route(
            "/api/images/{image_id}",
            get(get_image).delete(delete_image),
        )
        .route("/api/settings", get(get_settings))
        .route("/api/settings/{key}", put(put_setting))
        .route("/api/settings/test/pixiv", post(post_test_pixiv))
        .route("/api/settings/test/deepseek", post(post_test_deepseek))
        .route("/api/tasks", get(list_tasks))
        .route("/api/tasks/{task_id}", get(get_task))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers(Any),
        )
}

pub async fn serve(state: AppState, addr: SocketAddr) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve_listener(state, listener).await
}

pub async fn serve_listener(
    state: AppState,
    listener: tokio::net::TcpListener,
) -> Result<(), std::io::Error> {
    axum::serve(listener, router(state)).await
}
