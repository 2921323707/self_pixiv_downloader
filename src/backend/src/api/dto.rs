use serde::{Deserialize, Serialize};

use crate::accounts::PixivAccountRecord;
use crate::ai::{DeepSeekConnectionStatus, SmartParseInput, SmartParsePlan};
use crate::domain::{DownloadRequest, ImageCategory, ImageSource, R18Policy, TaskStatus, TaskType};
use crate::errors::{AppError, ErrorCode};
use crate::images::{
    ImageDeleteOutcome, ImageListQuery, ImageR18Visibility, ImageRecord, preview_url_for,
};
use crate::settings::{PublicSettingValue, SettingsRepository};
use crate::tasks::{
    AuthorDownloadRequest, BookmarkDownloadRequest, SmartDownloadRequest, TaskItemRecord,
    TaskListQuery, TaskLogRecord, TaskRecord,
};

use super::runtime::{
    DEFAULT_BATCH_COUNT, DEFAULT_DEEPSEEK_MODEL, DEFAULT_MAX_REQUEST_COUNT, setting_r18_policy_or,
    setting_u32_or,
};

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
pub(crate) struct TaskListParams {
    status: Option<String>,
    #[serde(rename = "type")]
    task_type: Option<String>,
    limit: Option<usize>,
    cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ImageListParams {
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
    pub user_uid: Option<String>,
    pub user_name: Option<String>,
    pub bound: bool,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeReadinessResponse {
    pub backend: RuntimeReadinessCheckResponse,
    pub pixiv_network: RuntimeReadinessCheckResponse,
    pub pixiv_account: RuntimePixivAccountReadinessResponse,
    pub deepseek: RuntimeReadinessCheckResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeReadinessCheckResponse {
    pub ok: bool,
    pub status: String,
    pub message: String,
    pub recommendation: Option<String>,
    pub action: Option<RuntimeReadinessActionResponse>,
    pub error_code: Option<String>,
    pub latency_ms: Option<u128>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimePixivAccountReadinessResponse {
    pub ok: bool,
    pub status: String,
    pub message: String,
    pub recommendation: Option<String>,
    pub action: Option<RuntimeReadinessActionResponse>,
    pub error_code: Option<String>,
    pub latency_ms: Option<u128>,
    pub account: Option<PixivAccountResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeReadinessActionResponse {
    pub label: String,
    pub href: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixivAccountResponse {
    pub user_uid: String,
    pub user_name: Option<String>,
    pub is_active: bool,
    pub last_verified_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixivAccountsListResponse {
    pub items: Vec<PixivAccountResponse>,
    pub active: Option<PixivAccountResponse>,
}

#[derive(Debug, Deserialize)]
pub struct PixivAccountActivateRequest {
    pub user_uid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PixivAccountDeleteResponse {
    pub deleted: bool,
}

impl SingleDownloadRequest {
    pub(crate) fn into_domain_request(self) -> Result<DownloadRequest, AppError> {
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
    pub(crate) fn into_author_request(
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
    pub(crate) fn into_bookmark_request(
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
    pub(crate) fn into_smart_parse_input(
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
    pub(crate) fn into_smart_download_request(
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
    pub(crate) fn into_repository_query(self) -> Result<TaskListQuery, AppError> {
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
    pub(crate) fn into_repository_query(self) -> Result<ImageListQuery, AppError> {
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

pub(crate) fn normalize_image_delete_ids(image_ids: Vec<String>) -> Result<Vec<String>, AppError> {
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

pub(crate) fn task_response(
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

pub(crate) fn task_summary_response(task: TaskRecord) -> TaskSummaryResponse {
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

pub(crate) fn image_summary_response(
    image: ImageRecord,
    tags: Vec<String>,
) -> ImageSummaryResponse {
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

pub(crate) fn image_detail_response(
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

pub(crate) fn image_delete_success_response(
    outcome: ImageDeleteOutcome,
) -> ImageDeleteItemResponse {
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

pub(crate) fn image_delete_error_response(
    image_id: &str,
    error: AppError,
) -> ImageDeleteItemResponse {
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

pub(crate) fn smart_parse_response(plan: SmartParsePlan) -> SmartParseResponse {
    SmartParseResponse {
        tags: plan.tags,
        negative_tags: plan.negative_tags,
        count_recommend: plan.count_recommend,
        r18_policy: plan.r18_policy.as_str().to_owned(),
        confidence: plan.confidence,
        model: plan.model,
    }
}

pub(crate) fn deepseek_connection_response(
    status: DeepSeekConnectionStatus,
) -> DeepSeekConnectionTestResponse {
    DeepSeekConnectionTestResponse {
        configured: status.configured,
        status: status.status,
        model: status.model,
    }
}

pub(crate) fn pixiv_account_response(account: PixivAccountRecord) -> PixivAccountResponse {
    PixivAccountResponse {
        user_uid: account.user_uid,
        user_name: account.user_name,
        is_active: account.is_active,
        last_verified_at: account.last_verified_at,
        created_at: account.created_at,
        updated_at: account.updated_at,
    }
}

pub(crate) fn setting_response(setting: PublicSettingValue) -> SettingResponse {
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
