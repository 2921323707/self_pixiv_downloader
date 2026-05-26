use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};

use crate::ai::{
    AiClient, DeepSeekConfig, DeepSeekConnectionStatus, DeepSeekHttpClient, SmartParseInput,
    SmartParsePlan,
};
use crate::db;
use crate::domain::{DownloadRequest, ImageCategory, ImageSource, R18Policy, TaskStatus, TaskType};
use crate::errors::{AppError, ErrorCode};
use crate::images::{
    ImageDeleteOutcome, ImageListQuery, ImageR18Visibility, ImageRecord, ImageRepository,
    delete_image_file_and_index, preview_url_for, resolve_image_file,
};
use crate::pixiv::PixivClient;
use crate::pixiv::http::PixivHttpClient;
use crate::settings::{PublicSettingValue, SettingsRepository};
use crate::storage::StoragePlanner;
use crate::tasks::{
    AuthorDownloadRequest, BookmarkDownloadRequest, SmartDownloadRequest, TaskItemRecord,
    TaskListQuery, TaskLogRecord, TaskRecord, TaskRepository, create_author_download_task,
    create_bookmark_download_task, create_single_download_task, create_smart_download_task,
    execute_queued_task,
};

const TASK_QUEUE_BUFFER: usize = 64;
const DEFAULT_DOWNLOAD_BASE_PATH: &str = "project:output";
const LEGACY_DEFAULT_DOWNLOAD_BASE_PATH: &str = "~/pixiv_downloads/";
const DEFAULT_BATCH_COUNT: u32 = 20;
const DEFAULT_MAX_REQUEST_COUNT: u32 = 100;
const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-v4-flash";

pub trait PixivClientFactory: Send + Sync {
    fn create(&self) -> Result<Box<dyn PixivClient>, AppError>;

    fn create_with_cookie(&self, cookie: Option<&str>) -> Result<Box<dyn PixivClient>, AppError> {
        let _ = cookie;
        self.create()
    }
}

pub trait AiClientFactory: Send + Sync {
    fn create(&self, config: DeepSeekConfig) -> Result<Box<dyn AiClient>, AppError>;
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    db_path: PathBuf,
    download_root: PathBuf,
    pixiv_client_factory: Arc<dyn PixivClientFactory>,
    ai_client_factory: Arc<dyn AiClientFactory>,
    task_sender: mpsc::Sender<QueuedTask>,
}

#[derive(Debug, Clone)]
struct QueuedTask {
    task_id: String,
}

impl AppState {
    pub fn new(
        db_path: impl Into<PathBuf>,
        download_root: impl Into<PathBuf>,
        pixiv_client_factory: Arc<dyn PixivClientFactory>,
    ) -> Self {
        let db_path = db_path.into();
        let download_root = download_root.into();
        let (task_sender, task_receiver) = mpsc::channel(TASK_QUEUE_BUFFER);
        spawn_worker(
            task_receiver,
            db_path.clone(),
            download_root.clone(),
            Arc::clone(&pixiv_client_factory),
        );

        Self {
            inner: Arc::new(AppStateInner {
                db_path,
                download_root,
                pixiv_client_factory,
                ai_client_factory: Arc::new(EnvAiClientFactory),
                task_sender,
            }),
        }
    }

    pub fn new_with_ai_factory(
        db_path: impl Into<PathBuf>,
        download_root: impl Into<PathBuf>,
        pixiv_client_factory: Arc<dyn PixivClientFactory>,
        ai_client_factory: Arc<dyn AiClientFactory>,
    ) -> Self {
        let state = Self::new(db_path, download_root, pixiv_client_factory);
        Self {
            inner: Arc::new(AppStateInner {
                db_path: state.inner.db_path.clone(),
                download_root: state.inner.download_root.clone(),
                pixiv_client_factory: Arc::clone(&state.inner.pixiv_client_factory),
                ai_client_factory,
                task_sender: state.inner.task_sender.clone(),
            }),
        }
    }

    pub fn from_env() -> Self {
        let download_root = env::var("PIXIV_DOWNLOAD_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_output_dir());
        let db_path = env::var("PIXIV_PLATFORM_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| download_root.join("pixiv_platform.sqlite3"));
        Self::new(db_path, download_root, Arc::new(EnvPixivClientFactory))
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.inner.db_path
    }

    pub fn download_root(&self) -> &PathBuf {
        &self.inner.download_root
    }
}

pub struct EnvPixivClientFactory;

impl PixivClientFactory for EnvPixivClientFactory {
    fn create(&self) -> Result<Box<dyn PixivClient>, AppError> {
        self.create_with_cookie(None)
    }

    fn create_with_cookie(&self, cookie: Option<&str>) -> Result<Box<dyn PixivClient>, AppError> {
        let cookie = cookie
            .map(str::to_owned)
            .or_else(|| env::var("PIXIV_PHPSESSID").ok())
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::MissingPixivCookie,
                    "Pixiv cookie is required in settings or PIXIV_PHPSESSID",
                )
            })?;
        Ok(Box::new(PixivHttpClient::new(cookie)?))
    }
}

pub struct EnvAiClientFactory;

impl AiClientFactory for EnvAiClientFactory {
    fn create(&self, config: DeepSeekConfig) -> Result<Box<dyn AiClient>, AppError> {
        Ok(Box::new(DeepSeekHttpClient::new(config)?))
    }
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    pub details: serde_json::Value,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    app_error: AppError,
}

impl ApiError {
    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            app_error: AppError::new(ErrorCode::TaskNotFound, message),
        }
    }
}

impl From<AppError> for ApiError {
    fn from(app_error: AppError) -> Self {
        let status = match app_error.code {
            ErrorCode::ValidationError
            | ErrorCode::MissingPixivCookie
            | ErrorCode::AiConfigMissing => StatusCode::BAD_REQUEST,
            ErrorCode::PixivAuthFailed => StatusCode::UNAUTHORIZED,
            ErrorCode::PixivForbidden => StatusCode::FORBIDDEN,
            ErrorCode::PixivNotFound | ErrorCode::TaskNotFound => StatusCode::NOT_FOUND,
            ErrorCode::PixivRateLimited => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::PixivNetworkError | ErrorCode::PixivParseError => StatusCode::BAD_GATEWAY,
            ErrorCode::FilesystemWriteFailed
            | ErrorCode::FilesystemPathCollision
            | ErrorCode::SqliteError
            | ErrorCode::AiParseFailed
            | ErrorCode::TaskCancelled
            | ErrorCode::R18PolicySkipped
            | ErrorCode::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self { status, app_error }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let body = ApiErrorEnvelope {
            error: ApiErrorBody {
                code: self.app_error.code.as_str().to_owned(),
                message: self.app_error.message,
                details: serde_json::json!({}),
            },
        };
        (self.status, Json(body)).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct SingleDownloadRequest {
    pub pixiv_id: String,
    pub page_index: Option<u32>,
    pub r18_policy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SingleDownloadResponse {
    pub task_id: String,
    pub image_id: Option<String>,
    pub download_status: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorDownloadApiRequest {
    pub author_uid: String,
    pub limit: Option<u32>,
    pub r18_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkDownloadApiRequest {
    pub limit: Option<u32>,
    pub r18_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SmartParseApiRequest {
    pub prompt: String,
    pub count: Option<u32>,
    pub r18_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SmartDownloadApiRequest {
    pub prompt: String,
    pub tags: Vec<String>,
    pub negative_tags: Option<Vec<String>>,
    pub count: Option<u32>,
    pub r18_policy: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmartParseResponse {
    pub tags: Vec<String>,
    pub negative_tags: Vec<String>,
    pub count_recommend: u32,
    pub r18_policy: String,
    pub confidence: f64,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchDownloadResponse {
    pub task_id: String,
    pub download_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub status: String,
    pub progress_total: Option<u32>,
    pub progress_done: u32,
    pub progress_failed: u32,
    pub current_item: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub updated_at: String,
    pub items: Vec<TaskItemResponse>,
    pub logs: Vec<TaskLogResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub items: Vec<TaskSummaryResponse>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSummaryResponse {
    pub task_id: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub status: String,
    pub progress_total: Option<u32>,
    pub progress_done: u32,
    pub progress_failed: u32,
    pub current_item: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskItemResponse {
    pub item_id: String,
    pub pixiv_id: Option<String>,
    pub page_index: Option<u32>,
    pub status: String,
    pub image_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskLogResponse {
    pub log_id: String,
    pub level: String,
    pub phase: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct TaskListParams {
    status: Option<String>,
    #[serde(rename = "type")]
    task_type: Option<String>,
    limit: Option<usize>,
    cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageListParams {
    tag: Option<String>,
    category: Option<String>,
    author_uid: Option<String>,
    source: Option<String>,
    r18_visibility: Option<String>,
    limit: Option<usize>,
    cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageListResponse {
    pub items: Vec<ImageSummaryResponse>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSummaryResponse {
    pub image_id: String,
    pub pixiv_id: String,
    pub page_index: u32,
    pub title: Option<String>,
    pub author_uid: Option<String>,
    pub tags: Vec<String>,
    pub category: String,
    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub downloaded_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDetailResponse {
    pub image_id: String,
    pub pixiv_id: String,
    pub page_index: u32,
    pub title: Option<String>,
    pub author_uid: Option<String>,
    pub tags: Vec<String>,
    pub sources: Vec<ImageSourceResponse>,
    pub category: String,
    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub map_x: Option<f64>,
    pub map_y: Option<f64>,
    pub downloaded_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSourceResponse {
    pub source: String,
    pub task_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageDeleteBatchRequest {
    pub image_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDeleteBatchResponse {
    pub items: Vec<ImageDeleteItemResponse>,
    pub deleted_count: u32,
    pub failed_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDeleteItemResponse {
    pub image_id: String,
    pub status: String,
    pub pixiv_id: Option<String>,
    pub page_index: Option<u32>,
    pub file_deleted: bool,
    pub file_missing: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsListResponse {
    pub items: Vec<SettingResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingResponse {
    pub key: String,
    pub value: serde_json::Value,
    pub is_secret: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SettingUpdateRequest {
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PixivConnectionTestRequest {
    pub pixiv_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixivConnectionTestResponse {
    pub configured: bool,
    pub status: String,
    pub pixiv_id: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepSeekConnectionTestResponse {
    pub configured: bool,
    pub status: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

async fn get_health() -> Json<ApiEnvelope<HealthResponse>> {
    Json(ApiEnvelope {
        data: HealthResponse {
            status: "ok".to_owned(),
        },
    })
}

async fn post_download_single(
    State(state): State<AppState>,
    Json(payload): Json<SingleDownloadRequest>,
) -> Result<(StatusCode, Json<ApiEnvelope<SingleDownloadResponse>>), ApiError> {
    let request = payload.into_domain_request()?;
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let factory = Arc::clone(&state.inner.pixiv_client_factory);
    let task_sender = state.inner.task_sender.clone();

    let task_id = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let runtime = resolve_runtime_settings(&conn, &download_root)?;
        fs::create_dir_all(&runtime.download_root)?;
        let _ = factory.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
        create_single_download_task(&request, &conn)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    if task_sender
        .send(QueuedTask {
            task_id: task_id.clone(),
        })
        .await
        .is_err()
    {
        let db_path_for_fail = state.inner.db_path.clone();
        let task_id_for_fail = task_id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            mark_task_failed(
                &db_path_for_fail,
                &task_id_for_fail,
                ErrorCode::InternalError.as_str(),
                "task queue is unavailable",
            )
        })
        .await;
        return Err(AppError::new(ErrorCode::InternalError, "task queue is unavailable").into());
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiEnvelope {
            data: SingleDownloadResponse {
                task_id,
                image_id: None,
                download_status: "pending".to_owned(),
            },
        }),
    ))
}

async fn post_smart_parse(
    State(state): State<AppState>,
    Json(payload): Json<SmartParseApiRequest>,
) -> Result<Json<ApiEnvelope<SmartParseResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let ai_factory = Arc::clone(&state.inner.ai_client_factory);
    let plan = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let settings = SettingsRepository::new(&conn);
        let config = resolve_deepseek_config(&settings)?;
        let input = payload.into_smart_parse_input(&conn)?;
        let client = ai_factory.create(config)?;
        client.parse_smart_prompt(&input)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope {
        data: smart_parse_response(plan),
    }))
}

async fn post_download_author(
    State(state): State<AppState>,
    Json(payload): Json<AuthorDownloadApiRequest>,
) -> Result<(StatusCode, Json<ApiEnvelope<BatchDownloadResponse>>), ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let factory = Arc::clone(&state.inner.pixiv_client_factory);
    let task_sender = state.inner.task_sender.clone();

    let task_id = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let runtime = resolve_runtime_settings(&conn, &download_root)?;
        fs::create_dir_all(&runtime.download_root)?;
        let _ = factory.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
        let request = payload.into_author_request(&conn)?;
        create_author_download_task(&request, &conn)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    if task_sender
        .send(QueuedTask {
            task_id: task_id.clone(),
        })
        .await
        .is_err()
    {
        let db_path_for_fail = state.inner.db_path.clone();
        let task_id_for_fail = task_id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            mark_task_failed(
                &db_path_for_fail,
                &task_id_for_fail,
                ErrorCode::InternalError.as_str(),
                "task queue is unavailable",
            )
        })
        .await;
        return Err(AppError::new(ErrorCode::InternalError, "task queue is unavailable").into());
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiEnvelope {
            data: BatchDownloadResponse {
                task_id,
                download_status: "pending".to_owned(),
            },
        }),
    ))
}

async fn post_download_bookmarks(
    State(state): State<AppState>,
    Json(payload): Json<BookmarkDownloadApiRequest>,
) -> Result<(StatusCode, Json<ApiEnvelope<BatchDownloadResponse>>), ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let factory = Arc::clone(&state.inner.pixiv_client_factory);
    let task_sender = state.inner.task_sender.clone();

    let task_id = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let runtime = resolve_runtime_settings(&conn, &download_root)?;
        fs::create_dir_all(&runtime.download_root)?;
        let _ = factory.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
        let request = payload.into_bookmark_request(&conn)?;
        create_bookmark_download_task(&request, &conn)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    if task_sender
        .send(QueuedTask {
            task_id: task_id.clone(),
        })
        .await
        .is_err()
    {
        let db_path_for_fail = state.inner.db_path.clone();
        let task_id_for_fail = task_id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            mark_task_failed(
                &db_path_for_fail,
                &task_id_for_fail,
                ErrorCode::InternalError.as_str(),
                "task queue is unavailable",
            )
        })
        .await;
        return Err(AppError::new(ErrorCode::InternalError, "task queue is unavailable").into());
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiEnvelope {
            data: BatchDownloadResponse {
                task_id,
                download_status: "pending".to_owned(),
            },
        }),
    ))
}

async fn post_smart_download(
    State(state): State<AppState>,
    Json(payload): Json<SmartDownloadApiRequest>,
) -> Result<(StatusCode, Json<ApiEnvelope<BatchDownloadResponse>>), ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let factory = Arc::clone(&state.inner.pixiv_client_factory);
    let task_sender = state.inner.task_sender.clone();

    let task_id = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let runtime = resolve_runtime_settings(&conn, &download_root)?;
        fs::create_dir_all(&runtime.download_root)?;
        let _ = factory.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
        let request = payload.into_smart_download_request(&conn)?;
        create_smart_download_task(&request, &conn)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    if task_sender
        .send(QueuedTask {
            task_id: task_id.clone(),
        })
        .await
        .is_err()
    {
        let db_path_for_fail = state.inner.db_path.clone();
        let task_id_for_fail = task_id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            mark_task_failed(
                &db_path_for_fail,
                &task_id_for_fail,
                ErrorCode::InternalError.as_str(),
                "task queue is unavailable",
            )
        })
        .await;
        return Err(AppError::new(ErrorCode::InternalError, "task queue is unavailable").into());
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiEnvelope {
            data: BatchDownloadResponse {
                task_id,
                download_status: "pending".to_owned(),
            },
        }),
    ))
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<TaskListParams>,
) -> Result<Json<ApiEnvelope<TaskListResponse>>, ApiError> {
    let query = params.into_repository_query()?;
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let page = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        TaskRepository::new(&conn).list_tasks(&query)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope {
        data: TaskListResponse {
            items: page.items.into_iter().map(task_summary_response).collect(),
            next_cursor: page.next_cursor_offset.map(|offset| offset.to_string()),
        },
    }))
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ApiEnvelope<TaskResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let task = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        let repo = TaskRepository::new(&conn);
        let task = repo
            .find_task(&task_id)?
            .ok_or_else(|| AppError::new(ErrorCode::TaskNotFound, "task not found"))?;
        let items = repo.items_for_task(&task.task_id)?;
        let logs = repo.logs_for_task(&task.task_id)?;
        Ok::<_, AppError>((task, items, logs))
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))?
    .map_err(|error| {
        if error.code == ErrorCode::TaskNotFound {
            ApiError::not_found(error.message)
        } else {
            ApiError::from(error)
        }
    })?;

    Ok(Json(ApiEnvelope {
        data: task_response(task.0, task.1, task.2),
    }))
}

async fn list_images(
    State(state): State<AppState>,
    Query(params): Query<ImageListParams>,
) -> Result<Json<ApiEnvelope<ImageListResponse>>, ApiError> {
    let query = params.into_repository_query()?;
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let page = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        ImageRepository::new(&conn).list_images(&query)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope {
        data: ImageListResponse {
            items: page
                .items
                .into_iter()
                .map(|item| image_summary_response(item.image, item.tags))
                .collect(),
            next_cursor: page.next_cursor_offset.map(|offset| offset.to_string()),
        },
    }))
}

async fn get_image(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
) -> Result<Json<ApiEnvelope<ImageDetailResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let detail = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        let repo = ImageRepository::new(&conn);
        let image = repo
            .find_by_id(&image_id)?
            .ok_or_else(|| AppError::new(ErrorCode::PixivNotFound, "image not found"))?;
        let tags = repo.tags_for_image(&image.image_id)?;
        let sources = repo.sources_for_image(&image.image_id)?;
        Ok::<_, AppError>((image, tags, sources))
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))?
    .map_err(|error| {
        if error.code == ErrorCode::PixivNotFound {
            ApiError {
                status: StatusCode::NOT_FOUND,
                app_error: error,
            }
        } else {
            ApiError::from(error)
        }
    })?;

    Ok(Json(ApiEnvelope {
        data: image_detail_response(detail.0, detail.1, detail.2),
    }))
}

async fn get_image_file(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
) -> Result<Response, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let file = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        let settings = SettingsRepository::new(&conn);
        let configured_root = resolve_download_root(&settings, &download_root)?;
        fs::create_dir_all(&configured_root)?;
        let repo = ImageRepository::new(&conn);
        let image = repo
            .find_by_id(&image_id)?
            .ok_or_else(|| AppError::new(ErrorCode::PixivNotFound, "image not found"))?;
        let file = resolve_image_file(&image, &[download_root, configured_root])?;
        let bytes = fs::read(&file.path).map_err(|_| {
            AppError::new(
                ErrorCode::FilesystemWriteFailed,
                "image file could not be read",
            )
        })?;
        Ok::<_, AppError>((file.content_type.to_owned(), bytes.len().to_string(), bytes))
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))?
    .map_err(|error| {
        if error.code == ErrorCode::PixivNotFound {
            ApiError {
                status: StatusCode::NOT_FOUND,
                app_error: error,
            }
        } else {
            ApiError::from(error)
        }
    })?;

    Ok((
        [
            (CONTENT_TYPE, file.0),
            (CONTENT_LENGTH, file.1),
            (CACHE_CONTROL, "private, max-age=60".to_owned()),
        ],
        file.2,
    )
        .into_response())
}

async fn delete_image(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
) -> Result<Json<ApiEnvelope<ImageDeleteItemResponse>>, ApiError> {
    let image_id = image_id.trim().to_owned();
    if image_id.is_empty() {
        return Err(AppError::validation("image_id cannot be empty").into());
    }

    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let item = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        let settings = SettingsRepository::new(&conn);
        let configured_root = resolve_download_root(&settings, &download_root)?;
        fs::create_dir_all(&configured_root)?;
        let repo = ImageRepository::new(&conn);
        delete_image_file_and_index(&repo, &image_id, &[download_root, configured_root])
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))?
    .map_err(|error| {
        if error.code == ErrorCode::PixivNotFound {
            ApiError {
                status: StatusCode::NOT_FOUND,
                app_error: error,
            }
        } else {
            ApiError::from(error)
        }
    })?;

    Ok(Json(ApiEnvelope {
        data: image_delete_success_response(item),
    }))
}

async fn post_delete_images(
    State(state): State<AppState>,
    Json(payload): Json<ImageDeleteBatchRequest>,
) -> Result<Json<ApiEnvelope<ImageDeleteBatchResponse>>, ApiError> {
    let image_ids = normalize_image_delete_ids(payload.image_ids)?;
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let items = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        let settings = SettingsRepository::new(&conn);
        let configured_root = resolve_download_root(&settings, &download_root)?;
        fs::create_dir_all(&configured_root)?;
        let repo = ImageRepository::new(&conn);
        let allowed_roots = [download_root, configured_root];

        Ok::<_, AppError>(
            image_ids
                .iter()
                .map(|image_id| {
                    match delete_image_file_and_index(&repo, image_id, &allowed_roots) {
                        Ok(outcome) => image_delete_success_response(outcome),
                        Err(error) => image_delete_error_response(image_id, error),
                    }
                })
                .collect::<Vec<_>>(),
        )
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    let deleted_count = items.iter().filter(|item| item.status == "deleted").count() as u32;
    let failed_count = items.len() as u32 - deleted_count;

    Ok(Json(ApiEnvelope {
        data: ImageDeleteBatchResponse {
            items,
            deleted_count,
            failed_count,
        },
    }))
}

async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<ApiEnvelope<SettingsListResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let settings = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        SettingsRepository::new(&conn).list_public()
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope {
        data: SettingsListResponse {
            items: settings.into_iter().map(setting_response).collect(),
        },
    }))
}

async fn put_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<SettingUpdateRequest>,
) -> Result<Json<ApiEnvelope<SettingResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let setting = tokio::task::spawn_blocking(move || {
        prepare_local_paths(&db_path, &download_root)?;
        let conn = db::open(&db_path)?;
        SettingsRepository::new(&conn).upsert_known_json(&key, &payload.value)
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope {
        data: setting_response(setting),
    }))
}

async fn post_test_pixiv(
    State(state): State<AppState>,
    Json(payload): Json<PixivConnectionTestRequest>,
) -> Result<Json<ApiEnvelope<PixivConnectionTestResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let factory = Arc::clone(&state.inner.pixiv_client_factory);
    let response = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let runtime = resolve_runtime_settings(&conn, &download_root)?;
        let pixiv = factory.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
        let pixiv_id = payload.pixiv_id.filter(|value| !value.trim().is_empty());

        if let Some(pixiv_id) = pixiv_id {
            if !pixiv_id.chars().all(|c| c.is_ascii_digit()) {
                return Err(AppError::validation("pixiv_id must contain only digits"));
            }
            let work = pixiv.fetch_work(&pixiv_id)?;
            Ok(PixivConnectionTestResponse {
                configured: true,
                status: "ok".to_owned(),
                pixiv_id: Some(pixiv_id),
                title: work.title,
            })
        } else {
            Ok(PixivConnectionTestResponse {
                configured: true,
                status: "configured".to_owned(),
                pixiv_id: None,
                title: None,
            })
        }
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope { data: response }))
}

async fn post_test_deepseek(
    State(state): State<AppState>,
) -> Result<Json<ApiEnvelope<DeepSeekConnectionTestResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let ai_factory = Arc::clone(&state.inner.ai_client_factory);
    let response = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let settings = SettingsRepository::new(&conn);
        let config = resolve_deepseek_config(&settings)?;
        let client = ai_factory.create(config)?;
        client.test_connection()
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope {
        data: deepseek_connection_response(response),
    }))
}

fn prepare_local_paths(db_path: &PathBuf, download_root: &PathBuf) -> Result<(), AppError> {
    fs::create_dir_all(download_root)?;
    prepare_db_path(db_path)
}

fn prepare_db_path(db_path: &PathBuf) -> Result<(), AppError> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct RuntimeSettings {
    download_root: PathBuf,
    pixiv_cookie: Option<String>,
}

fn resolve_runtime_settings(
    conn: &rusqlite::Connection,
    fallback_download_root: &FsPath,
) -> Result<RuntimeSettings, AppError> {
    let settings = SettingsRepository::new(conn);
    Ok(RuntimeSettings {
        download_root: resolve_download_root(&settings, fallback_download_root)?,
        pixiv_cookie: resolve_pixiv_cookie(&settings)?,
    })
}

fn resolve_pixiv_cookie(settings: &SettingsRepository<'_>) -> Result<Option<String>, AppError> {
    let Some(setting) = settings.get_raw("pixiv_cookie")? else {
        return Ok(None);
    };
    let value = setting_string(&setting.value_json, "pixiv_cookie")?;
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "***" {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_owned()))
    }
}

fn resolve_deepseek_config(settings: &SettingsRepository<'_>) -> Result<DeepSeekConfig, AppError> {
    let api_key = resolve_secret_setting(settings, "deepseek_api_key")?
        .or_else(|| env::var("DEEPSEEK_API_KEY").ok())
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::AiConfigMissing,
                "DeepSeek API key is required in settings or DEEPSEEK_API_KEY",
            )
        })?;
    let base_url =
        resolve_string_setting(settings, "deepseek_base_url", DEFAULT_DEEPSEEK_BASE_URL)?;
    let model = resolve_string_setting(settings, "deepseek_model", DEFAULT_DEEPSEEK_MODEL)?;

    Ok(DeepSeekConfig {
        api_key,
        base_url,
        model,
    })
}

fn resolve_secret_setting(
    settings: &SettingsRepository<'_>,
    key: &str,
) -> Result<Option<String>, AppError> {
    let Some(setting) = settings.get_raw(key)? else {
        return Ok(None);
    };
    let value = setting_string(&setting.value_json, key)?;
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "***" {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_owned()))
    }
}

fn resolve_string_setting(
    settings: &SettingsRepository<'_>,
    key: &str,
    fallback: &str,
) -> Result<String, AppError> {
    let Some(setting) = settings.get_raw(key)? else {
        return Ok(fallback.to_owned());
    };
    let value = setting_string(&setting.value_json, key)?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Ok(fallback.to_owned())
    } else {
        Ok(trimmed.to_owned())
    }
}

fn resolve_download_root(
    settings: &SettingsRepository<'_>,
    fallback_download_root: &FsPath,
) -> Result<PathBuf, AppError> {
    let Some(setting) = settings.get_raw("download_base_path")? else {
        return Ok(fallback_download_root.to_path_buf());
    };
    let raw = setting_string(&setting.value_json, "download_base_path")?;
    let trimmed = raw.trim();
    if trimmed == DEFAULT_DOWNLOAD_BASE_PATH || trimmed == LEGACY_DEFAULT_DOWNLOAD_BASE_PATH {
        return Ok(fallback_download_root.to_path_buf());
    }
    expand_download_root(trimmed)
}

fn expand_download_root(value: &str) -> Result<PathBuf, AppError> {
    if value.is_empty() {
        return Err(AppError::validation("download_base_path cannot be empty"));
    }
    if value.contains('\0') {
        return Err(AppError::validation(
            "download_base_path cannot contain NUL bytes",
        ));
    }

    let path = if value == "~" {
        PathBuf::from(home_dir()?)
    } else if let Some(rest) = value.strip_prefix("~/") {
        PathBuf::from(home_dir()?).join(rest)
    } else {
        PathBuf::from(value)
    };

    if !path.is_absolute() {
        return Err(AppError::validation(
            "download_base_path must be absolute, start with ~, or be project:output",
        ));
    }
    Ok(path)
}

fn home_dir() -> Result<String, AppError> {
    env::var("HOME").map_err(|_| {
        AppError::validation("HOME is required to expand download_base_path starting with ~")
    })
}

fn setting_string(raw_json: &str, key: &str) -> Result<String, AppError> {
    serde_json::from_str::<String>(raw_json)
        .map_err(|_| AppError::validation(format!("{key} must be stored as a JSON string")))
}

fn setting_u32_or(
    settings: &SettingsRepository<'_>,
    key: &str,
    fallback: u32,
) -> Result<u32, AppError> {
    let Some(setting) = settings.get_raw(key)? else {
        return Ok(fallback);
    };
    let value = serde_json::from_str::<u64>(&setting.value_json)
        .map_err(|_| AppError::validation(format!("{key} must be stored as a JSON number")))?;
    u32::try_from(value).map_err(|_| AppError::validation(format!("{key} is too large")))
}

fn setting_r18_policy_or(
    settings: &SettingsRepository<'_>,
    fallback: R18Policy,
) -> Result<R18Policy, AppError> {
    let Some(setting) = settings.get_raw("r18_policy")? else {
        return Ok(fallback);
    };
    let value = setting_string(&setting.value_json, "r18_policy")?;
    R18Policy::from_api(&value).ok_or_else(|| AppError::validation("r18_policy is invalid"))
}

fn project_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(FsPath::parent)
        .map(|root| root.join("output"))
        .unwrap_or_else(|| PathBuf::from("output"))
}

fn spawn_worker(
    mut task_receiver: mpsc::Receiver<QueuedTask>,
    db_path: PathBuf,
    download_root: PathBuf,
    pixiv_client_factory: Arc<dyn PixivClientFactory>,
) {
    tokio::spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            let task_id = task.task_id;
            let db_path_for_job = db_path.clone();
            let download_root_for_job = download_root.clone();
            let factory_for_job = Arc::clone(&pixiv_client_factory);
            let task_id_for_job = task_id.clone();

            let result = tokio::task::spawn_blocking(move || {
                prepare_db_path(&db_path_for_job)?;
                let conn = db::open(&db_path_for_job)?;
                let runtime = resolve_runtime_settings(&conn, &download_root_for_job)?;
                fs::create_dir_all(&runtime.download_root)?;
                let storage = StoragePlanner::new(runtime.download_root);
                let pixiv = factory_for_job.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
                execute_queued_task(&task_id_for_job, pixiv.as_ref(), &storage, &conn)
            })
            .await;

            match result {
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    let _ =
                        mark_task_failed(&db_path, &task_id, error.code.as_str(), &error.message);
                }
                Err(error) => {
                    let _ = mark_task_failed(
                        &db_path,
                        &task_id,
                        ErrorCode::InternalError.as_str(),
                        &error.to_string(),
                    );
                }
            }
        }
    });
}

fn mark_task_failed(
    db_path: &PathBuf,
    task_id: &str,
    error_code: &str,
    error_message: &str,
) -> Result<(), AppError> {
    let conn = db::open(db_path)?;
    let repo = TaskRepository::new(&conn);
    if repo
        .find_task(task_id)?
        .is_some_and(|task| task.status.is_terminal())
    {
        return Ok(());
    }
    repo.fail_task(task_id, error_code, error_message)
}

impl SingleDownloadRequest {
    fn into_domain_request(self) -> Result<DownloadRequest, AppError> {
        let pixiv_id = self.pixiv_id.trim().to_owned();
        if pixiv_id.is_empty() {
            return Err(AppError::validation("pixiv_id cannot be empty"));
        }
        if !pixiv_id.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::validation("pixiv_id must contain only digits"));
        }

        let r18_policy = match self.r18_policy {
            Some(value) => R18Policy::from_api(&value)
                .ok_or_else(|| AppError::validation("r18_policy is invalid"))?,
            None => R18Policy::Exclude,
        };

        Ok(DownloadRequest {
            pixiv_id,
            page_index: self.page_index,
            source: ImageSource::Single,
            r18_policy,
        })
    }
}

impl AuthorDownloadApiRequest {
    fn into_author_request(
        self,
        conn: &rusqlite::Connection,
    ) -> Result<AuthorDownloadRequest, AppError> {
        let author_uid = self.author_uid.trim().to_owned();
        if author_uid.is_empty() {
            return Err(AppError::validation("author_uid cannot be empty"));
        }
        if !author_uid.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::validation("author_uid must contain only digits"));
        }

        let settings = SettingsRepository::new(conn);
        let max_request_count =
            setting_u32_or(&settings, "max_request_count", DEFAULT_MAX_REQUEST_COUNT)?;
        if max_request_count == 0 {
            return Err(AppError::validation("max_request_count must be at least 1"));
        }
        let default_batch_count =
            setting_u32_or(&settings, "default_batch_count", DEFAULT_BATCH_COUNT)?;
        let limit = self
            .limit
            .unwrap_or(default_batch_count)
            .min(max_request_count);
        if limit == 0 {
            return Err(AppError::validation("limit must be at least 1"));
        }
        if self
            .limit
            .is_some_and(|requested| requested > max_request_count)
        {
            return Err(AppError::validation(
                "limit cannot exceed max_request_count",
            ));
        }

        let r18_policy = match self.r18_policy {
            Some(value) => R18Policy::from_api(&value)
                .ok_or_else(|| AppError::validation("r18_policy is invalid"))?,
            None => setting_r18_policy_or(&settings, R18Policy::Exclude)?,
        };

        Ok(AuthorDownloadRequest {
            author_uid,
            limit,
            r18_policy,
        })
    }
}

impl BookmarkDownloadApiRequest {
    fn into_bookmark_request(
        self,
        conn: &rusqlite::Connection,
    ) -> Result<BookmarkDownloadRequest, AppError> {
        let settings = SettingsRepository::new(conn);
        let max_request_count =
            setting_u32_or(&settings, "max_request_count", DEFAULT_MAX_REQUEST_COUNT)?;
        if max_request_count == 0 {
            return Err(AppError::validation("max_request_count must be at least 1"));
        }
        let default_batch_count =
            setting_u32_or(&settings, "default_batch_count", DEFAULT_BATCH_COUNT)?;
        let limit = self
            .limit
            .unwrap_or(default_batch_count)
            .min(max_request_count);
        if limit == 0 {
            return Err(AppError::validation("limit must be at least 1"));
        }
        if self
            .limit
            .is_some_and(|requested| requested > max_request_count)
        {
            return Err(AppError::validation(
                "limit cannot exceed max_request_count",
            ));
        }

        let r18_policy = match self.r18_policy {
            Some(value) => R18Policy::from_api(&value)
                .ok_or_else(|| AppError::validation("r18_policy is invalid"))?,
            None => setting_r18_policy_or(&settings, R18Policy::Exclude)?,
        };

        Ok(BookmarkDownloadRequest { limit, r18_policy })
    }
}

impl SmartParseApiRequest {
    fn into_smart_parse_input(
        self,
        conn: &rusqlite::Connection,
    ) -> Result<SmartParseInput, AppError> {
        let prompt = self.prompt.trim().to_owned();
        if prompt.is_empty() {
            return Err(AppError::validation("prompt cannot be empty"));
        }
        if prompt.chars().count() > 2000 {
            return Err(AppError::validation("prompt cannot exceed 2000 characters"));
        }

        let settings = SettingsRepository::new(conn);
        let max_count = setting_u32_or(&settings, "max_request_count", DEFAULT_MAX_REQUEST_COUNT)?;
        if max_count == 0 {
            return Err(AppError::validation("max_request_count must be at least 1"));
        }
        let default_count = setting_u32_or(&settings, "default_batch_count", DEFAULT_BATCH_COUNT)?;
        let count_hint = self.count.unwrap_or(default_count).min(max_count);
        if count_hint == 0 {
            return Err(AppError::validation("count must be at least 1"));
        }
        if self.count.is_some_and(|requested| requested > max_count) {
            return Err(AppError::validation(
                "count cannot exceed max_request_count",
            ));
        }

        let r18_policy = match self.r18_policy {
            Some(value) => R18Policy::from_api(&value)
                .ok_or_else(|| AppError::validation("r18_policy is invalid"))?,
            None => setting_r18_policy_or(&settings, R18Policy::Exclude)?,
        };

        Ok(SmartParseInput {
            prompt,
            count_hint,
            max_count,
            r18_policy,
        })
    }
}

impl SmartDownloadApiRequest {
    fn into_smart_download_request(
        self,
        conn: &rusqlite::Connection,
    ) -> Result<SmartDownloadRequest, AppError> {
        let prompt = self.prompt.trim().to_owned();
        if prompt.is_empty() {
            return Err(AppError::validation("prompt cannot be empty"));
        }
        if prompt.chars().count() > 2000 {
            return Err(AppError::validation("prompt cannot exceed 2000 characters"));
        }

        let tags = normalize_api_tags(self.tags);
        if tags.is_empty() {
            return Err(AppError::validation("tags cannot be empty"));
        }
        let negative_tags = normalize_api_tags(self.negative_tags.unwrap_or_default());

        let settings = SettingsRepository::new(conn);
        let max_count = setting_u32_or(&settings, "max_request_count", DEFAULT_MAX_REQUEST_COUNT)?;
        if max_count == 0 {
            return Err(AppError::validation("max_request_count must be at least 1"));
        }
        let default_count = setting_u32_or(&settings, "default_batch_count", DEFAULT_BATCH_COUNT)?;
        let limit = self.count.unwrap_or(default_count).min(max_count);
        if limit == 0 {
            return Err(AppError::validation("count must be at least 1"));
        }
        if self.count.is_some_and(|requested| requested > max_count) {
            return Err(AppError::validation(
                "count cannot exceed max_request_count",
            ));
        }

        let r18_policy = match self.r18_policy {
            Some(value) => R18Policy::from_api(&value)
                .ok_or_else(|| AppError::validation("r18_policy is invalid"))?,
            None => setting_r18_policy_or(&settings, R18Policy::Exclude)?,
        };
        let model = self
            .model
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_DEEPSEEK_MODEL.to_owned());

        Ok(SmartDownloadRequest {
            prompt,
            tags,
            negative_tags,
            limit,
            r18_policy,
            model,
        })
    }
}

fn normalize_api_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }
        if !normalized.iter().any(|existing: &String| existing == tag) {
            normalized.push(tag.to_owned());
        }
        if normalized.len() == 12 {
            break;
        }
    }
    normalized
}

impl TaskListParams {
    fn into_repository_query(self) -> Result<TaskListQuery, AppError> {
        let status = self
            .status
            .as_deref()
            .map(|value| {
                TaskStatus::from_db(value)
                    .ok_or_else(|| AppError::validation("task status filter is invalid"))
            })
            .transpose()?;
        let task_type = self
            .task_type
            .as_deref()
            .map(|value| {
                TaskType::from_db(value)
                    .ok_or_else(|| AppError::validation("task type filter is invalid"))
            })
            .transpose()?;

        Ok(TaskListQuery {
            status,
            task_type,
            limit: self.limit.unwrap_or(20).clamp(1, 100),
            cursor_offset: parse_offset_cursor(self.cursor)?,
        })
    }
}

impl ImageListParams {
    fn into_repository_query(self) -> Result<ImageListQuery, AppError> {
        let category = self
            .category
            .as_deref()
            .map(|value| {
                ImageCategory::from_db(value)
                    .ok_or_else(|| AppError::validation("image category filter is invalid"))
            })
            .transpose()?;
        let source = self
            .source
            .as_deref()
            .map(|value| {
                ImageSource::from_db(value)
                    .ok_or_else(|| AppError::validation("image source filter is invalid"))
            })
            .transpose()?;
        let r18_visibility = match self.r18_visibility.as_deref().unwrap_or("exclude") {
            "include" => ImageR18Visibility::Include,
            "exclude" => ImageR18Visibility::Exclude,
            "only_r18" => ImageR18Visibility::OnlyR18,
            _ => return Err(AppError::validation("r18_visibility filter is invalid")),
        };

        Ok(ImageListQuery {
            tag: self.tag,
            category,
            author_uid: self.author_uid,
            source,
            r18_visibility,
            limit: self.limit.unwrap_or(24).clamp(1, 100),
            cursor_offset: parse_offset_cursor(self.cursor)?,
        })
    }
}

fn normalize_image_delete_ids(image_ids: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut normalized = Vec::new();
    for image_id in image_ids {
        let trimmed = image_id.trim();
        if trimmed.is_empty() {
            return Err(AppError::validation(
                "image_ids cannot contain empty values",
            ));
        }
        if !normalized
            .iter()
            .any(|existing: &String| existing == trimmed)
        {
            normalized.push(trimmed.to_owned());
        }
    }

    if normalized.is_empty() {
        return Err(AppError::validation("image_ids cannot be empty"));
    }
    if normalized.len() > 100 {
        return Err(AppError::validation(
            "image_ids cannot contain more than 100 items",
        ));
    }

    Ok(normalized)
}

fn parse_offset_cursor(cursor: Option<String>) -> Result<usize, AppError> {
    match cursor.as_deref().filter(|value| !value.trim().is_empty()) {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| AppError::validation("cursor is invalid")),
        None => Ok(0),
    }
}

fn task_response(
    task: TaskRecord,
    items: Vec<TaskItemRecord>,
    logs: Vec<TaskLogRecord>,
) -> TaskResponse {
    TaskResponse {
        task_id: task.task_id,
        task_type: task.task_type.as_str().to_owned(),
        status: task.status.as_str().to_owned(),
        progress_total: task.progress_total,
        progress_done: task.progress_done,
        progress_failed: task.progress_failed,
        current_item: task.current_item,
        error_code: task.error_code,
        error_message: task.error_message,
        created_at: task.created_at,
        started_at: task.started_at,
        finished_at: task.finished_at,
        updated_at: task.updated_at,
        items: items.into_iter().map(task_item_response).collect(),
        logs: logs.into_iter().map(task_log_response).collect(),
    }
}

fn task_summary_response(task: TaskRecord) -> TaskSummaryResponse {
    TaskSummaryResponse {
        task_id: task.task_id,
        task_type: task.task_type.as_str().to_owned(),
        status: task.status.as_str().to_owned(),
        progress_total: task.progress_total,
        progress_done: task.progress_done,
        progress_failed: task.progress_failed,
        current_item: task.current_item,
        error_code: task.error_code,
        error_message: task.error_message,
        created_at: task.created_at,
        started_at: task.started_at,
        finished_at: task.finished_at,
        updated_at: task.updated_at,
    }
}

fn task_item_response(item: TaskItemRecord) -> TaskItemResponse {
    TaskItemResponse {
        item_id: item.item_id,
        pixiv_id: item.pixiv_id,
        page_index: item.page_index,
        status: item.status.as_str().to_owned(),
        image_id: item.image_id,
        error_code: item.error_code,
        error_message: item.error_message,
        created_at: item.created_at,
        updated_at: item.updated_at,
    }
}

fn image_summary_response(image: ImageRecord, tags: Vec<String>) -> ImageSummaryResponse {
    let image_url = preview_url_for(&image.image_id);
    ImageSummaryResponse {
        image_id: image.image_id,
        preview_url: Some(image_url.clone()),
        thumbnail_url: Some(image_url),
        pixiv_id: image.pixiv_id,
        page_index: image.page_index,
        title: image.title,
        author_uid: image.author_uid,
        tags,
        category: image.category.as_str().to_owned(),
        width: image.width,
        height: image.height,
        downloaded_at: image.downloaded_at,
        created_at: image.created_at,
    }
}

fn image_detail_response(
    image: ImageRecord,
    tags: Vec<String>,
    sources: Vec<crate::images::ImageSourceRecord>,
) -> ImageDetailResponse {
    let image_url = preview_url_for(&image.image_id);
    ImageDetailResponse {
        image_id: image.image_id,
        preview_url: Some(image_url.clone()),
        thumbnail_url: Some(image_url),
        pixiv_id: image.pixiv_id,
        page_index: image.page_index,
        title: image.title,
        author_uid: image.author_uid,
        tags,
        sources: sources
            .into_iter()
            .map(|source| ImageSourceResponse {
                source: source.source.as_str().to_owned(),
                task_id: source.task_id,
                created_at: source.created_at,
            })
            .collect(),
        category: image.category.as_str().to_owned(),
        width: image.width,
        height: image.height,
        map_x: image.map_x,
        map_y: image.map_y,
        downloaded_at: image.downloaded_at,
        created_at: image.created_at,
        updated_at: image.updated_at,
    }
}

fn image_delete_success_response(outcome: ImageDeleteOutcome) -> ImageDeleteItemResponse {
    ImageDeleteItemResponse {
        image_id: outcome.image_id,
        status: "deleted".to_owned(),
        pixiv_id: Some(outcome.pixiv_id),
        page_index: Some(outcome.page_index),
        file_deleted: outcome.file_deleted,
        file_missing: outcome.file_missing,
        error_code: None,
        error_message: None,
    }
}

fn image_delete_error_response(image_id: &str, error: AppError) -> ImageDeleteItemResponse {
    ImageDeleteItemResponse {
        image_id: image_id.to_owned(),
        status: if error.code == ErrorCode::PixivNotFound {
            "not_found".to_owned()
        } else {
            "failed".to_owned()
        },
        pixiv_id: None,
        page_index: None,
        file_deleted: false,
        file_missing: false,
        error_code: Some(error.code.as_str().to_owned()),
        error_message: Some(error.message),
    }
}

fn smart_parse_response(plan: SmartParsePlan) -> SmartParseResponse {
    SmartParseResponse {
        tags: plan.tags,
        negative_tags: plan.negative_tags,
        count_recommend: plan.count_recommend,
        r18_policy: plan.r18_policy.as_str().to_owned(),
        confidence: plan.confidence,
        model: plan.model,
    }
}

fn deepseek_connection_response(
    status: DeepSeekConnectionStatus,
) -> DeepSeekConnectionTestResponse {
    DeepSeekConnectionTestResponse {
        configured: status.configured,
        status: status.status,
        model: status.model,
    }
}

fn setting_response(setting: PublicSettingValue) -> SettingResponse {
    SettingResponse {
        key: setting.key,
        value: serde_json::from_str(&setting.value_json)
            .unwrap_or(serde_json::Value::String(setting.value_json)),
        is_secret: setting.is_secret,
        updated_at: setting.updated_at,
    }
}

fn task_log_response(log: TaskLogRecord) -> TaskLogResponse {
    TaskLogResponse {
        log_id: log.log_id,
        level: log.level.as_str().to_owned(),
        phase: log.phase,
        message: log.message,
        context: log.context_json.map(|raw| {
            serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::Value::String(raw))
        }),
        created_at: log.created_at,
    }
}

#[cfg(test)]
mod tests {
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
        AiClientFactory, ApiEnvelope, AppState, BatchDownloadResponse,
        DeepSeekConnectionTestResponse, ImageDeleteBatchResponse, ImageListResponse,
        PixivClientFactory, PixivConnectionTestResponse, SettingResponse, SettingsListResponse,
        SingleDownloadResponse, SmartParseResponse, TaskListResponse, TaskResponse, router,
    };
    use crate::db;
    use crate::domain::{
        ImageCategory, ImageSource, PixivPage, PixivWork, PixivWorkRef, R18Policy,
    };
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

        fn create_with_cookie(
            &self,
            cookie: Option<&str>,
        ) -> Result<Box<dyn PixivClient>, AppError> {
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
        let envelope: ApiEnvelope<ImageDeleteBatchResponse> =
            serde_json::from_slice(&body).unwrap();
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
        let envelope: ApiEnvelope<ImageDeleteBatchResponse> =
            serde_json::from_str(&body_text).unwrap();
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
        let envelope: ApiEnvelope<PixivConnectionTestResponse> =
            serde_json::from_slice(&body).unwrap();
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
}
