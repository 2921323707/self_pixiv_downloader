use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::ai::{AiClient, DeepSeekConfig, DeepSeekHttpClient};
use crate::errors::{AppError, ErrorCode};
use crate::pixiv::PixivClient;
use crate::pixiv::http::PixivHttpClient;

pub mod dto;
pub mod error;
pub(crate) mod handlers;
pub mod routes;
pub(crate) mod runtime;
pub(crate) mod worker;

pub use dto::*;
pub use error::{ApiEnvelope, ApiError, ApiErrorBody, ApiErrorEnvelope};
pub use routes::{router, serve, serve_listener};

const TASK_QUEUE_BUFFER: usize = 64;

pub trait PixivClientFactory: Send + Sync {
    fn create(&self) -> Result<Box<dyn PixivClient>, AppError>;

    fn probe_network(&self) -> Result<(), AppError> {
        Ok(())
    }

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
pub(crate) struct QueuedTask {
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
        worker::spawn_worker(
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
            .unwrap_or_else(|_| runtime::default_download_root());
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
    fn probe_network(&self) -> Result<(), AppError> {
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()?
            .get("https://www.pixiv.net/")
            .header(reqwest::header::REFERER, "https://www.pixiv.net/")
            .send()
            .map(|_| ())
            .map_err(AppError::from)
    }

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

#[cfg(test)]
mod tests;
