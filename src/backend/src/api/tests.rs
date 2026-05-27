use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, mpsc as std_mpsc};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request};
use tokio::time::sleep;
use tower::ServiceExt;

use crate::ai::{
    AiClient, DeepSeekConfig, DeepSeekConnectionStatus, SmartParseInput, SmartParsePlan,
};
use crate::api::{
    AiClientFactory, ApiEnvelope, AppState, BatchDownloadResponse, DeepSeekConnectionTestResponse,
    ImageDeleteBatchResponse, ImageListResponse, PixivClientFactory, PixivConnectionTestResponse,
    SettingResponse, SettingsListResponse, SingleDownloadResponse, SmartParseResponse,
    TaskListResponse, TaskResponse, router,
};
use crate::db;
use crate::domain::{ImageCategory, ImageSource, PixivPage, PixivWork, PixivWorkRef, R18Policy};
use crate::errors::{AppError, ErrorCode};
use crate::images::{ImageRepository, NewImageRecord};
use crate::pixiv::PixivClient;
use crate::settings::SettingsRepository;

#[derive(Clone)]
struct StaticPixivFactory {
    work: PixivWork,
    author_works: Vec<PixivWorkRef>,
    bookmarks: Vec<PixivWorkRef>,
    tag_searches: HashMap<String, Vec<PixivWorkRef>>,
    images: HashMap<String, Vec<u8>>,
    required_cookie: Option<String>,
}

impl PixivClientFactory for StaticPixivFactory {
    fn create(&self) -> Result<Box<dyn PixivClient>, AppError> {
        self.create_with_cookie(None)
    }

    fn create_with_cookie(&self, cookie: Option<&str>) -> Result<Box<dyn PixivClient>, AppError> {
        if let Some(required) = self.required_cookie.as_deref() {
            if cookie != Some(required) {
                return Err(AppError::new(
                    ErrorCode::MissingPixivCookie,
                    "mock Pixiv cookie was not provided from settings",
                ));
            }
        }
        Ok(Box::new(StaticPixivClient {
            work: self.work.clone(),
            author_works: self.author_works.clone(),
            bookmarks: self.bookmarks.clone(),
            tag_searches: self.tag_searches.clone(),
            images: self.images.clone(),
        }))
    }
}

struct StaticPixivClient {
    work: PixivWork,
    author_works: Vec<PixivWorkRef>,
    bookmarks: Vec<PixivWorkRef>,
    tag_searches: HashMap<String, Vec<PixivWorkRef>>,
    images: HashMap<String, Vec<u8>>,
}

#[derive(Clone)]
struct StaticAiFactory {
    plan: SmartParsePlan,
    required_key: Option<String>,
}

impl AiClientFactory for StaticAiFactory {
    fn create(&self, config: DeepSeekConfig) -> Result<Box<dyn AiClient>, AppError> {
        if let Some(required) = self.required_key.as_deref() {
            if config.api_key != required {
                return Err(AppError::new(
                    ErrorCode::AiConfigMissing,
                    "mock DeepSeek key was not provided from settings",
                ));
            }
        }
        Ok(Box::new(StaticAiClient {
            plan: self.plan.clone(),
            model: config.model,
        }))
    }
}

struct StaticAiClient {
    plan: SmartParsePlan,
    model: String,
}

impl AiClient for StaticAiClient {
    fn parse_smart_prompt(&self, input: &SmartParseInput) -> Result<SmartParsePlan, AppError> {
        let mut plan = self.plan.clone();
        plan.count_recommend = plan.count_recommend.clamp(1, input.max_count);
        plan.r18_policy = input.r18_policy;
        plan.model = self.model.clone();
        Ok(plan)
    }

    fn test_connection(&self) -> Result<DeepSeekConnectionStatus, AppError> {
        Ok(DeepSeekConnectionStatus {
            configured: true,
            status: "ok".to_owned(),
            model: self.model.clone(),
        })
    }
}

impl PixivClient for StaticPixivClient {
    fn fetch_work(&self, pixiv_id: &str) -> Result<PixivWork, AppError> {
        if pixiv_id == self.work.pixiv_id {
            Ok(self.work.clone())
        } else {
            Err(AppError::new(
                ErrorCode::PixivNotFound,
                format!("Pixiv work {pixiv_id} not found"),
            ))
        }
    }

    fn download_image(&self, url: &str) -> Result<Vec<u8>, AppError> {
        self.images.get(url).cloned().ok_or_else(|| {
            AppError::new(
                ErrorCode::PixivNetworkError,
                format!("No mock bytes for {url}"),
            )
        })
    }

    fn fetch_author_works(
        &self,
        author_uid: &str,
        limit: u32,
    ) -> Result<Vec<PixivWorkRef>, AppError> {
        if author_uid == self.work.author_uid.as_deref().unwrap_or_default() {
            Ok(self
                .author_works
                .iter()
                .take(limit as usize)
                .cloned()
                .collect())
        } else {
            Err(AppError::new(
                ErrorCode::PixivNotFound,
                format!("Pixiv author {author_uid} not found"),
            ))
        }
    }

    fn fetch_bookmarks(
        &self,
        limit: u32,
        _r18_policy: R18Policy,
    ) -> Result<Vec<PixivWorkRef>, AppError> {
        Ok(self
            .bookmarks
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    fn search_works_by_tags(
        &self,
        tags: &[String],
        _negative_tags: &[String],
        limit: u32,
        _r18_policy: R18Policy,
    ) -> Result<Vec<PixivWorkRef>, AppError> {
        let works = self
            .tag_searches
            .get(&tags.join("\n"))
            .cloned()
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::PixivNotFound,
                    format!("Pixiv tag search {:?} not found", tags),
                )
            })?;
        Ok(works.into_iter().take(limit as usize).collect())
    }
}

#[derive(Clone)]
struct BlockingPixivFactory {
    work: PixivWork,
    images: HashMap<String, Vec<u8>>,
    gate: Arc<Mutex<Option<std_mpsc::Receiver<()>>>>,
}

impl PixivClientFactory for BlockingPixivFactory {
    fn create(&self) -> Result<Box<dyn PixivClient>, AppError> {
        Ok(Box::new(BlockingPixivClient {
            work: self.work.clone(),
            images: self.images.clone(),
            gate: Arc::clone(&self.gate),
        }))
    }
}

struct BlockingPixivClient {
    work: PixivWork,
    images: HashMap<String, Vec<u8>>,
    gate: Arc<Mutex<Option<std_mpsc::Receiver<()>>>>,
}

impl PixivClient for BlockingPixivClient {
    fn fetch_work(&self, pixiv_id: &str) -> Result<PixivWork, AppError> {
        if pixiv_id == self.work.pixiv_id {
            Ok(self.work.clone())
        } else {
            Err(AppError::new(
                ErrorCode::PixivNotFound,
                format!("Pixiv work {pixiv_id} not found"),
            ))
        }
    }

    fn download_image(&self, url: &str) -> Result<Vec<u8>, AppError> {
        if let Some(receiver) = self.gate.lock().unwrap().take() {
            receiver.recv().map_err(|error| {
                AppError::new(
                    ErrorCode::InternalError,
                    format!("blocking test gate closed: {error}"),
                )
            })?;
        }

        self.images.get(url).cloned().ok_or_else(|| {
            AppError::new(
                ErrorCode::PixivNetworkError,
                format!("No mock bytes for {url}"),
            )
        })
    }

    fn fetch_author_works(
        &self,
        author_uid: &str,
        limit: u32,
    ) -> Result<Vec<PixivWorkRef>, AppError> {
        if author_uid == self.work.author_uid.as_deref().unwrap_or_default() {
            Ok(vec![PixivWorkRef {
                pixiv_id: self.work.pixiv_id.clone(),
            }]
            .into_iter()
            .take(limit as usize)
            .collect())
        } else {
            Err(AppError::new(
                ErrorCode::PixivNotFound,
                format!("Pixiv author {author_uid} not found"),
            ))
        }
    }

    fn fetch_bookmarks(
        &self,
        limit: u32,
        _r18_policy: R18Policy,
    ) -> Result<Vec<PixivWorkRef>, AppError> {
        Ok(vec![PixivWorkRef {
            pixiv_id: self.work.pixiv_id.clone(),
        }]
        .into_iter()
        .take(limit as usize)
        .collect())
    }

    fn search_works_by_tags(
        &self,
        _tags: &[String],
        _negative_tags: &[String],
        limit: u32,
        _r18_policy: R18Policy,
    ) -> Result<Vec<PixivWorkRef>, AppError> {
        Ok(vec![PixivWorkRef {
            pixiv_id: self.work.pixiv_id.clone(),
        }]
        .into_iter()
        .take(limit as usize)
        .collect())
    }
}

fn sample_work() -> PixivWork {
    PixivWork {
        pixiv_id: "123456".to_owned(),
        title: Some("api mock".to_owned()),
        author_uid: Some("9988".to_owned()),
        author_name: Some("mock author".to_owned()),
        tags: vec!["cyan".to_owned()],
        category: ImageCategory::Normal,
        pages: vec![PixivPage {
            page_index: 0,
            original_url: "https://i.pximg.net/img-original/mock/123456_p0.jpg".to_owned(),
            width: Some(1200),
            height: Some(1800),
            extension: Some("jpg".to_owned()),
        }],
    }
}

fn test_state(name: &str) -> (AppState, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("api.sqlite3");
    let mut images = HashMap::new();
    images.insert(
        "https://i.pximg.net/img-original/mock/123456_p0.jpg".to_owned(),
        b"fake api image bytes".to_vec(),
    );
    (
        AppState::new(
            &db_path,
            &root,
            Arc::new(StaticPixivFactory {
                work: sample_work(),
                author_works: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                bookmarks: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                tag_searches: HashMap::from([(
                    "blue hair\ncyberpunk".to_owned(),
                    vec![PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    }],
                )]),
                images,
                required_cookie: None,
            }),
        ),
        root,
    )
}

fn blocking_state(name: &str) -> (AppState, std::path::PathBuf, std_mpsc::Sender<()>) {
    let root = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("api.sqlite3");
    let mut images = HashMap::new();
    images.insert(
        "https://i.pximg.net/img-original/mock/123456_p0.jpg".to_owned(),
        b"fake api image bytes".to_vec(),
    );
    let (sender, receiver) = std_mpsc::channel();
    (
        AppState::new(
            &db_path,
            &root,
            Arc::new(BlockingPixivFactory {
                work: sample_work(),
                images,
                gate: Arc::new(Mutex::new(Some(receiver))),
            }),
        ),
        root,
        sender,
    )
}

fn failing_state(name: &str) -> (AppState, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("api.sqlite3");
    (
        AppState::new(
            &db_path,
            &root,
            Arc::new(StaticPixivFactory {
                work: sample_work(),
                author_works: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                bookmarks: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                tag_searches: HashMap::from([(
                    "blue hair\ncyberpunk".to_owned(),
                    vec![PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    }],
                )]),
                images: HashMap::new(),
                required_cookie: None,
            }),
        ),
        root,
    )
}

fn cookie_required_state(name: &str) -> (AppState, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("api.sqlite3");
    let mut images = HashMap::new();
    images.insert(
        "https://i.pximg.net/img-original/mock/123456_p0.jpg".to_owned(),
        b"fake api image bytes".to_vec(),
    );
    (
        AppState::new(
            &db_path,
            &root.join("startup-root"),
            Arc::new(StaticPixivFactory {
                work: sample_work(),
                author_works: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                bookmarks: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                tag_searches: HashMap::from([(
                    "blue hair\ncyberpunk".to_owned(),
                    vec![PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    }],
                )]),
                images,
                required_cookie: Some("placeholder-runtime-value".to_owned()),
            }),
        ),
        root,
    )
}

fn smart_state(name: &str) -> (AppState, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let db_path = root.join("api.sqlite3");
    (
        AppState::new_with_ai_factory(
            &db_path,
            &root,
            Arc::new(StaticPixivFactory {
                work: sample_work(),
                author_works: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                bookmarks: vec![PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                }],
                tag_searches: HashMap::from([(
                    "blue hair\ncyberpunk".to_owned(),
                    vec![PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    }],
                )]),
                images: HashMap::new(),
                required_cookie: None,
            }),
            Arc::new(StaticAiFactory {
                plan: SmartParsePlan {
                    tags: vec!["blue hair".to_owned(), "cyberpunk".to_owned()],
                    negative_tags: vec!["low quality".to_owned()],
                    count_recommend: 20,
                    r18_policy: R18Policy::Exclude,
                    confidence: 0.86,
                    model: "deepseek-v4-flash".to_owned(),
                },
                required_key: Some("placeholder-runtime-value".to_owned()),
            }),
        ),
        root,
    )
}

#[tokio::test]
async fn api_health_returns_ok() {
    let (state, root) = test_state("health");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<super::HealthResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope.data.status, "ok");

    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_001_req_task_001_post_single_download_enqueues_and_returns_task_id() {
    let (state, root, release_download) = blocking_state("post_single");
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"pixiv_id":"123456","page_index":0,"r18_policy":"exclude"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<SingleDownloadResponse> = serde_json::from_slice(&body).unwrap();
    assert!(!envelope.data.task_id.is_empty());
    assert_eq!(envelope.data.download_status, "pending");
    assert_eq!(envelope.data.image_id, None);

    let queued = get_task_snapshot(app.clone(), &envelope.data.task_id).await;
    assert!(queued.status == "pending" || queued.status == "running");
    assert_eq!(queued.progress_done, 0);

    release_download.send(()).unwrap();
    let completed = poll_task_status(app, &envelope.data.task_id, "completed").await;
    assert_eq!(
        completed.items[0].image_id.as_deref(),
        Some("pixiv:123456:p0")
    );
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_task_002_req_task_004_get_task_returns_items_and_logs() {
    let (state, root) = test_state("get_task");
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<SingleDownloadResponse> = serde_json::from_slice(&body).unwrap();

    let envelope = poll_task_status(app, &created.data.task_id, "completed").await;

    assert_eq!(envelope.status, "completed");
    assert_eq!(envelope.progress_done, 1);
    assert_eq!(envelope.items.len(), 1);
    assert!(envelope.logs.iter().any(|log| log.phase == "finish_task"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_task_004_queued_single_download_preserves_failure_diagnostics() {
    let (state, root) = failing_state("queue_failure");
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<SingleDownloadResponse> = serde_json::from_slice(&body).unwrap();

    let failed = poll_task_status(app, &created.data.task_id, "failed").await;
    assert_eq!(failed.progress_failed, 1);
    assert_eq!(failed.error_code.as_deref(), Some("PIXIV_NETWORK_ERROR"));
    assert_eq!(failed.items[0].status, "item_failed");
    assert!(
        failed
            .logs
            .iter()
            .any(|log| log.level == "error" && log.phase == "finish_task")
    );
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_ui_002_post_single_download_rejects_invalid_pixiv_id() {
    let (state, root) = test_state("validation");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"abc"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "VALIDATION_ERROR");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_002_req_ui_005_get_images_returns_gallery_metadata() {
    let (state, root) = test_state("list_images");
    seed_image(state.db_path(), &root);
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images?tag=cyan&source=single&r18_visibility=exclude")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains(root.to_string_lossy().as_ref()));
    let envelope: ApiEnvelope<ImageListResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope.data.items.len(), 1);
    assert_eq!(envelope.data.items[0].image_id, "image-api-1");
    assert_eq!(envelope.data.items[0].tags, vec!["cyan", "girl"]);
    assert_eq!(
        envelope.data.items[0].preview_url.as_deref(),
        Some("/api/images/image-api-1/file")
    );

    let detail = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/image-api-1")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), axum::http::StatusCode::OK);
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_004_req_sec_002_get_image_file_serves_bytes_without_path_leak() {
    let (state, root) = test_state("image_file");
    seed_image(state.db_path(), &root);
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/image-api-1/file")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .unwrap(),
        "image/jpeg"
    );
    assert_eq!(
        response
            .headers()
            .get(axum::http::header::CONTENT_LENGTH)
            .unwrap(),
        "18"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body.as_ref(), b"seeded image bytes");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_004_get_image_file_returns_404_for_missing_file() {
    let (state, root) = test_state("image_file_missing");
    seed_image(state.db_path(), &root);
    fs::remove_file(root.join("seeded.jpg")).unwrap();
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/image-api-1/file")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains(root.to_string_lossy().as_ref()));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_004_get_image_file_returns_404_for_unknown_image() {
    let (state, root) = test_state("image_file_unknown");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/missing-image/file")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_004_req_sec_002_get_image_file_rejects_outside_allowed_root() {
    let (state, root) = test_state("image_file_unsafe");
    let outside = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_outside_{}.jpg",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(&outside, b"outside image bytes").unwrap();
    {
        let conn = db::open(state.db_path()).unwrap();
        let repo = ImageRepository::new(&conn);
        repo.insert(&NewImageRecord {
            image_id: "image-outside".to_owned(),
            pixiv_id: "987654".to_owned(),
            page_index: 0,
            author_uid: Some("9988".to_owned()),
            title: Some("outside image".to_owned()),
            category: ImageCategory::Normal,
            local_path: outside.to_string_lossy().to_string(),
            thumbnail_path: None,
            width: Some(100),
            height: Some(100),
            map_x: None,
            map_y: None,
            downloaded_at: "2026-05-22T00:00:00Z".to_owned(),
        })
        .unwrap();
    }
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/image-outside/file")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains(outside.to_string_lossy().as_ref()));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(outside);
}

#[tokio::test]
async fn req_img_007_delete_image_removes_file_and_index_without_path_leak() {
    let (state, root) = test_state("delete_image");
    seed_image(state.db_path(), &root);
    let image_path = root.join("seeded.jpg");
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/images/image-api-1")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains(root.to_string_lossy().as_ref()));
    assert!(!image_path.exists());

    let detail = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/image-api-1")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), axum::http::StatusCode::NOT_FOUND);
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_007_delete_batch_returns_per_item_results() {
    let (state, root) = test_state("delete_batch");
    seed_image(state.db_path(), &root);
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/images/delete-batch")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"image_ids":["image-api-1","missing-image"]}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<ImageDeleteBatchResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope.data.deleted_count, 1);
    assert_eq!(envelope.data.failed_count, 1);
    assert_eq!(envelope.data.items[0].status, "deleted");
    assert_eq!(envelope.data.items[1].status, "not_found");
    assert_eq!(
        envelope.data.items[1].error_code.as_deref(),
        Some("PIXIV_NOT_FOUND")
    );

    let list = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images?r18_visibility=exclude")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(list.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<ImageListResponse> = serde_json::from_slice(&body).unwrap();
    assert!(envelope.data.items.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_img_007_req_sec_002_delete_batch_rejects_unsafe_file_path_per_item() {
    let (state, root) = test_state("delete_batch_unsafe");
    let outside = std::env::temp_dir().join(format!(
        "pixiv_platform_backend_api_delete_outside_{}.jpg",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(&outside, b"outside image bytes").unwrap();
    {
        let conn = db::open(state.db_path()).unwrap();
        let repo = ImageRepository::new(&conn);
        repo.insert(&NewImageRecord {
            image_id: "image-outside".to_owned(),
            pixiv_id: "987654".to_owned(),
            page_index: 0,
            author_uid: Some("9988".to_owned()),
            title: Some("outside image".to_owned()),
            category: ImageCategory::Normal,
            local_path: outside.to_string_lossy().to_string(),
            thumbnail_path: None,
            width: Some(100),
            height: Some(100),
            map_x: None,
            map_y: None,
            downloaded_at: "2026-05-22T00:00:00Z".to_owned(),
        })
        .unwrap();
    }
    let app = router(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/images/delete-batch")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"image_ids":["image-outside"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains(outside.to_string_lossy().as_ref()));
    let envelope: ApiEnvelope<ImageDeleteBatchResponse> = serde_json::from_str(&body_text).unwrap();
    assert_eq!(envelope.data.deleted_count, 0);
    assert_eq!(envelope.data.failed_count, 1);
    assert_eq!(envelope.data.items[0].status, "failed");
    assert_eq!(
        envelope.data.items[0].error_code.as_deref(),
        Some("VALIDATION_ERROR")
    );
    assert!(outside.exists());

    let detail = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/images/image-outside")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), axum::http::StatusCode::OK);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(outside);
}

#[tokio::test]
async fn req_task_002_req_ui_003_get_tasks_returns_task_list() {
    let (state, root) = test_state("list_tasks");
    let app = router(state);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<SingleDownloadResponse> = serde_json::from_slice(&body).unwrap();
    let _ = poll_task_status(app.clone(), &created.data.task_id, "completed").await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/tasks?type=single&limit=10")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<TaskListResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope.data.items.len(), 1);
    assert_eq!(envelope.data.items[0].task_id, created.data.task_id);
    assert_eq!(envelope.data.items[0].status, "completed");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_cfg_001_req_sec_001_settings_api_lists_and_saves_masked_values() {
    let (state, root) = test_state("settings");
    fs::create_dir_all(&root).unwrap();
    {
        let conn = db::open(state.db_path()).unwrap();
        SettingsRepository::new(&conn)
            .upsert("deepseek_api_key", "\"placeholder-runtime-value\"", true)
            .unwrap();
    }
    let app = router(state);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/settings")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), axum::http::StatusCode::OK);
    let body = to_bytes(list.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<SettingsListResponse> = serde_json::from_slice(&body).unwrap();
    let api_key = envelope
        .data
        .items
        .iter()
        .find(|setting| setting.key == "deepseek_api_key")
        .unwrap();
    assert_eq!(api_key.value, serde_json::json!("***"));

    let save = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/api/settings/r18_policy")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"value":"include_blurred"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(save.status(), axum::http::StatusCode::OK);
    let body = to_bytes(save.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<SettingResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope.data.value, serde_json::json!("include_blurred"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_001_req_cfg_002_single_download_uses_settings_cookie_and_download_root() {
    let (state, root) = cookie_required_state("settings_backed_download");
    let configured_root = root.join("configured-downloads");
    let startup_root = root.join("startup-root");
    let app = router(state);

    save_setting(
        app.clone(),
        "pixiv_cookie",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;
    save_setting(
        app.clone(),
        "download_base_path",
        serde_json::json!(configured_root.to_string_lossy()),
    )
    .await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<SingleDownloadResponse> = serde_json::from_slice(&body).unwrap();

    let completed = poll_task_status(app, &created.data.task_id, "completed").await;
    assert_eq!(
        completed.items[0].image_id.as_deref(),
        Some("pixiv:123456:p0")
    );
    assert!(
        configured_root
            .join("originals/123456/123456_p0.jpg")
            .exists()
    );
    assert!(!startup_root.join("originals/123456/123456_p0.jpg").exists());
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_007_settings_backed_pixiv_cookie_is_required_before_enqueue() {
    let (state, root) = cookie_required_state("missing_settings_cookie");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/download/single")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "MISSING_PIXIV_COOKIE");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_007_settings_pixiv_test_uses_masked_cookie_without_download() {
    let (state, root) = cookie_required_state("pixiv_test");
    let app = router(state);
    save_setting(
        app.clone(),
        "pixiv_cookie",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/settings/test/pixiv")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"pixiv_id":"123456"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<PixivConnectionTestResponse> = serde_json::from_slice(&body).unwrap();
    assert!(envelope.data.configured);
    assert_eq!(envelope.data.status, "ok");
    assert_eq!(envelope.data.title.as_deref(), Some("api mock"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_cfg_003_settings_deepseek_test_uses_masked_key_without_exposing_secret() {
    let (state, root) = smart_state("deepseek_test");
    let app = router(state);
    save_setting(
        app.clone(),
        "deepseek_api_key",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/settings/test/deepseek")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains("placeholder-runtime-value"));
    let envelope: ApiEnvelope<DeepSeekConnectionTestResponse> =
        serde_json::from_str(&body_text).unwrap();
    assert!(envelope.data.configured);
    assert_eq!(envelope.data.status, "ok");
    assert_eq!(envelope.data.model, "deepseek-v4-flash");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_ai_001_post_smart_parse_returns_structured_tag_plan() {
    let (state, root) = smart_state("smart_parse");
    let app = router(state);
    save_setting(
        app.clone(),
        "deepseek_api_key",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;
    save_setting(app.clone(), "default_batch_count", serde_json::json!(12)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/smart/parse")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"prompt":"下载蓝发赛博朋克少女","r18_policy":"exclude"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let envelope: ApiEnvelope<SmartParseResponse> = serde_json::from_slice(&body).unwrap();
    assert_eq!(envelope.data.tags, vec!["blue hair", "cyberpunk"]);
    assert_eq!(envelope.data.negative_tags, vec!["low quality"]);
    assert_eq!(envelope.data.r18_policy, "exclude");
    assert_eq!(envelope.data.model, "deepseek-v4-flash");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_ai_005_post_smart_parse_requires_deepseek_key_before_parse() {
    let (state, root) = smart_state("smart_parse_missing_key");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/smart/parse")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"prompt":"blue hair"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "AI_CONFIG_MISSING");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_cfg_004_post_smart_parse_rejects_count_above_max_request_count() {
    let (state, root) = smart_state("smart_parse_count");
    let app = router(state);
    save_setting(
        app.clone(),
        "deepseek_api_key",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;
    save_setting(app.clone(), "max_request_count", serde_json::json!(5)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/smart/parse")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"prompt":"blue hair","count":6}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "VALIDATION_ERROR");
    assert!(error.error.message.contains("max_request_count"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_ai_002_post_smart_download_enqueues_tag_search_task() {
    let (state, root) = cookie_required_state("smart_download");
    let app = router(state);
    save_setting(
        app.clone(),
        "pixiv_cookie",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;
    save_setting(app.clone(), "default_batch_count", serde_json::json!(1)).await;

    let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/smart/download")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"prompt":"下载蓝发赛博朋克少女","tags":["blue hair","cyberpunk"],"negative_tags":["low quality"],"r18_policy":"exclude","model":"deepseek-v4-flash"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<BatchDownloadResponse> = serde_json::from_slice(&body).unwrap();
    let completed = poll_task_status(app, &created.data.task_id, "completed").await;
    assert_eq!(completed.task_type, "smart");
    assert_eq!(completed.progress_total, Some(1));
    assert_eq!(completed.progress_done, 1);
    assert_eq!(completed.items[0].status, "saved");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_007_post_smart_download_requires_pixiv_cookie_before_enqueue() {
    let (state, root) = cookie_required_state("smart_missing_cookie");
    let app = router(state);

    let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/smart/download")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"prompt":"blue hair","tags":["blue hair"],"count":1,"r18_policy":"exclude"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "MISSING_PIXIV_COOKIE");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_cfg_004_post_smart_download_rejects_count_above_max_request_count() {
    let (state, root) = test_state("smart_download_limit");
    let app = router(state);
    save_setting(app.clone(), "max_request_count", serde_json::json!(1)).await;

    let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/smart/download")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"prompt":"blue hair","tags":["blue hair"],"count":2,"r18_policy":"exclude"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "VALIDATION_ERROR");
    assert!(error.error.message.contains("max_request_count"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_003_post_author_download_enqueues_and_uses_default_batch_count() {
    let (state, root) = cookie_required_state("author_download");
    let app = router(state);
    save_setting(
        app.clone(),
        "pixiv_cookie",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;
    save_setting(app.clone(), "default_batch_count", serde_json::json!(1)).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/downloads/author")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"author_uid":"9988"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<BatchDownloadResponse> = serde_json::from_slice(&body).unwrap();
    let completed = poll_task_status(app, &created.data.task_id, "completed").await;
    assert_eq!(completed.task_type, "author");
    assert_eq!(completed.progress_total, Some(1));
    assert_eq!(completed.progress_done, 1);
    assert_eq!(completed.items.len(), 1);
    assert_eq!(completed.items[0].status, "saved");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_002_post_bookmark_download_enqueues_and_uses_default_batch_count() {
    let (state, root) = cookie_required_state("bookmark_download");
    let app = router(state);
    save_setting(
        app.clone(),
        "pixiv_cookie",
        serde_json::json!("placeholder-runtime-value"),
    )
    .await;
    save_setting(app.clone(), "default_batch_count", serde_json::json!(1)).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/downloads/bookmarks")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::ACCEPTED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created: ApiEnvelope<BatchDownloadResponse> = serde_json::from_slice(&body).unwrap();
    let completed = poll_task_status(app, &created.data.task_id, "completed").await;
    assert_eq!(completed.task_type, "bookmark");
    assert_eq!(completed.progress_total, Some(1));
    assert_eq!(completed.progress_done, 1);
    assert_eq!(completed.items.len(), 1);
    assert_eq!(completed.items[0].status, "saved");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_cfg_004_post_author_download_rejects_limit_above_max_request_count() {
    let (state, root) = test_state("author_limit");
    let app = router(state);
    save_setting(app.clone(), "max_request_count", serde_json::json!(1)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/downloads/author")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"author_uid":"9988","limit":2,"r18_policy":"exclude"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "VALIDATION_ERROR");
    assert!(error.error.message.contains("max_request_count"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_cfg_004_post_bookmark_download_rejects_limit_above_max_request_count() {
    let (state, root) = test_state("bookmark_limit");
    let app = router(state);
    save_setting(app.clone(), "max_request_count", serde_json::json!(1)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/downloads/bookmarks")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"limit":2,"r18_policy":"exclude"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "VALIDATION_ERROR");
    assert!(error.error.message.contains("max_request_count"));
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_007_post_author_download_requires_pixiv_cookie_before_enqueue() {
    let (state, root) = cookie_required_state("author_missing_cookie");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/downloads/author")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"author_uid":"9988","limit":1,"r18_policy":"exclude"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "MISSING_PIXIV_COOKIE");
    let _ = fs::remove_dir_all(root);
}

#[tokio::test]
async fn req_dl_007_post_bookmark_download_requires_pixiv_cookie_before_enqueue() {
    let (state, root) = cookie_required_state("bookmark_missing_cookie");
    let app = router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/downloads/bookmarks")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"limit":1,"r18_policy":"exclude"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: crate::api::ApiErrorEnvelope = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "MISSING_PIXIV_COOKIE");
    let _ = fs::remove_dir_all(root);
}

async fn get_task_snapshot(app: Router, task_id: &str) -> TaskResponse {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/api/tasks/{task_id}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<ApiEnvelope<TaskResponse>>(&body)
        .unwrap()
        .data
}

async fn poll_task_status(app: Router, task_id: &str, expected_status: &str) -> TaskResponse {
    for _ in 0..50 {
        let task = get_task_snapshot(app.clone(), task_id).await;
        if task.status == expected_status {
            return task;
        }
        sleep(Duration::from_millis(20)).await;
    }

    let task = get_task_snapshot(app, task_id).await;
    panic!(
        "task {} did not reach status {}; latest status was {}",
        task_id, expected_status, task.status
    );
}

async fn save_setting(app: Router, key: &str, value: serde_json::Value) -> SettingResponse {
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/api/settings/{key}"))
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({ "value": value }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<ApiEnvelope<SettingResponse>>(&body)
        .unwrap()
        .data
}

fn seed_image(db_path: &std::path::Path, root: &std::path::Path) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("seeded.jpg"), b"seeded image bytes").unwrap();
    let conn = db::open(db_path).unwrap();
    let repo = ImageRepository::new(&conn);
    repo.insert(&NewImageRecord {
        image_id: "image-api-1".to_owned(),
        pixiv_id: "123456".to_owned(),
        page_index: 0,
        author_uid: Some("9988".to_owned()),
        title: Some("seeded api image".to_owned()),
        category: ImageCategory::Normal,
        local_path: root.join("seeded.jpg").to_string_lossy().to_string(),
        thumbnail_path: None,
        width: Some(1200),
        height: Some(1800),
        map_x: None,
        map_y: None,
        downloaded_at: "2026-05-22T00:00:00Z".to_owned(),
    })
    .unwrap();
    repo.replace_tags("image-api-1", &["cyan".to_owned(), "girl".to_owned()])
        .unwrap();
    repo.add_source("image-api-1", ImageSource::Single, None)
        .unwrap();
}
