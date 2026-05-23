use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, ToSql, params, params_from_iter};
use serde_json::Value;

use crate::domain::{
    DownloadItemStatus, DownloadRequest, ImageSource, R18Policy, TaskItemStatus, TaskLogLevel,
    TaskStatus, TaskType,
};
use crate::downloads::{DownloadRepositoryContext, download_single_with_db};
use crate::errors::{AppError, ErrorCode};
use crate::images::ImageRepository;
use crate::pixiv::PixivClient;
use crate::storage::StoragePlanner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTaskRecord {
    pub task_id: String,
    pub task_type: TaskType,
    pub request_json: String,
    pub progress_total: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    pub task_id: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub request_json: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTaskLogRecord {
    pub log_id: String,
    pub task_id: String,
    pub level: TaskLogLevel,
    pub phase: String,
    pub message: String,
    pub context_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskLogRecord {
    pub log_id: String,
    pub task_id: String,
    pub level: TaskLogLevel,
    pub phase: String,
    pub message: String,
    pub context_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTaskItemRecord {
    pub item_id: String,
    pub task_id: String,
    pub pixiv_id: Option<String>,
    pub page_index: Option<u32>,
    pub status: TaskItemStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItemRecord {
    pub item_id: String,
    pub task_id: String,
    pub pixiv_id: Option<String>,
    pub page_index: Option<u32>,
    pub status: TaskItemStatus,
    pub image_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListQuery {
    pub status: Option<TaskStatus>,
    pub task_type: Option<TaskType>,
    pub limit: usize,
    pub cursor_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListPage {
    pub items: Vec<TaskRecord>,
    pub next_cursor_offset: Option<usize>,
}

pub struct TaskRepository<'conn> {
    conn: &'conn Connection,
}

impl<'conn> TaskRepository<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    pub fn insert_task(&self, task: &NewTaskRecord) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO tasks (
                task_id, type, status, request_json, progress_total, progress_done,
                progress_failed, created_at, updated_at
             ) VALUES (
                ?1, ?2, 'pending', ?3, ?4, 0,
                0, datetime('now'), datetime('now')
             )",
            params![
                task.task_id,
                task.task_type.as_str(),
                task.request_json,
                task.progress_total
            ],
        )?;
        Ok(())
    }

    pub fn find_task(&self, task_id: &str) -> Result<Option<TaskRecord>, AppError> {
        self.conn
            .query_row(
                "SELECT
                    task_id, type, status, request_json, progress_total, progress_done,
                    progress_failed, current_item, error_code, error_message,
                    created_at, started_at, finished_at, updated_at
                 FROM tasks
                 WHERE task_id = ?1",
                params![task_id],
                row_to_task,
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn list_tasks(&self, query: &TaskListQuery) -> Result<TaskListPage, AppError> {
        let limit = query.limit.clamp(1, 100);
        let fetch_limit = limit + 1;
        let mut sql = String::from(
            "SELECT
                task_id, type, status, request_json, progress_total, progress_done,
                progress_failed, current_item, error_code, error_message,
                created_at, started_at, finished_at, updated_at
             FROM tasks
             WHERE 1 = 1",
        );
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(status) = query.status {
            sql.push_str(" AND status = ?");
            values.push(Box::new(status.as_str().to_owned()));
        }

        if let Some(task_type) = query.task_type {
            sql.push_str(" AND type = ?");
            values.push(Box::new(task_type.as_str().to_owned()));
        }

        sql.push_str(" ORDER BY created_at DESC, task_id DESC LIMIT ? OFFSET ?");
        values.push(Box::new(fetch_limit as i64));
        values.push(Box::new(query.cursor_offset as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let tasks = stmt
            .query_map(
                params_from_iter(values.iter().map(|value| value.as_ref() as &dyn ToSql)),
                row_to_task,
            )?
            .collect::<Result<Vec<_>, _>>()?;
        let has_next = tasks.len() > limit;

        Ok(TaskListPage {
            items: tasks.into_iter().take(limit).collect(),
            next_cursor_offset: has_next.then_some(query.cursor_offset + limit),
        })
    }

    pub fn start_task(&self, task_id: &str, current_item: Option<&str>) -> Result<(), AppError> {
        let affected = self.conn.execute(
            "UPDATE tasks
             SET status = 'running',
                 started_at = COALESCE(started_at, datetime('now')),
                 current_item = ?2,
                 updated_at = datetime('now')
             WHERE task_id = ?1 AND status = 'pending'",
            params![task_id, current_item],
        )?;
        ensure_updated(affected, "task is not pending")
    }

    pub fn update_progress(
        &self,
        task_id: &str,
        progress_done: u32,
        progress_failed: u32,
        current_item: Option<&str>,
    ) -> Result<(), AppError> {
        let task = self
            .find_task(task_id)?
            .ok_or_else(|| AppError::new(ErrorCode::InternalError, "task does not exist"))?;
        if task.status.is_terminal() {
            return Err(AppError::validation(
                "terminal task progress cannot be changed",
            ));
        }
        if progress_done < task.progress_done || progress_failed < task.progress_failed {
            return Err(AppError::validation("task progress cannot decrease"));
        }
        if let Some(total) = task.progress_total {
            if progress_done + progress_failed > total {
                return Err(AppError::validation("task progress cannot exceed total"));
            }
        }

        self.conn.execute(
            "UPDATE tasks
             SET progress_done = ?2,
                 progress_failed = ?3,
                 current_item = ?4,
                 updated_at = datetime('now')
             WHERE task_id = ?1",
            params![task_id, progress_done, progress_failed, current_item],
        )?;
        Ok(())
    }

    pub fn set_progress_total(&self, task_id: &str, progress_total: u32) -> Result<(), AppError> {
        let affected = self.conn.execute(
            "UPDATE tasks
             SET progress_total = ?2,
                 updated_at = datetime('now')
             WHERE task_id = ?1 AND status IN ('pending', 'running')",
            params![task_id, progress_total],
        )?;
        ensure_updated(affected, "task is already terminal")
    }

    pub fn complete_task(&self, task_id: &str) -> Result<(), AppError> {
        self.finish_task(task_id, TaskStatus::Completed, None, None)
    }

    pub fn fail_task(
        &self,
        task_id: &str,
        error_code: &str,
        error_message: &str,
    ) -> Result<(), AppError> {
        self.finish_task(
            task_id,
            TaskStatus::Failed,
            Some(error_code),
            Some(error_message),
        )
    }

    pub fn finish_task(
        &self,
        task_id: &str,
        status: TaskStatus,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), AppError> {
        if !status.is_terminal() {
            return Err(AppError::validation("task finish status must be terminal"));
        }

        let affected = self.conn.execute(
            "UPDATE tasks
             SET status = ?2,
                 error_code = ?3,
                 error_message = ?4,
                 finished_at = COALESCE(finished_at, datetime('now')),
                 current_item = NULL,
                 updated_at = datetime('now')
             WHERE task_id = ?1 AND status IN ('pending', 'running')",
            params![task_id, status.as_str(), error_code, error_message],
        )?;
        ensure_updated(affected, "task is already terminal")
    }

    pub fn insert_item(&self, item: &NewTaskItemRecord) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO task_items (
                item_id, task_id, pixiv_id, page_index, status, created_at, updated_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now')
             )",
            params![
                item.item_id,
                item.task_id,
                item.pixiv_id,
                item.page_index,
                item.status.as_str()
            ],
        )?;
        Ok(())
    }

    pub fn update_item_status(
        &self,
        item_id: &str,
        status: TaskItemStatus,
        image_id: Option<&str>,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), AppError> {
        let affected = self.conn.execute(
            "UPDATE task_items
             SET status = ?2,
                 image_id = ?3,
                 error_code = ?4,
                 error_message = ?5,
                 updated_at = datetime('now')
             WHERE item_id = ?1",
            params![
                item_id,
                status.as_str(),
                image_id,
                error_code,
                error_message
            ],
        )?;
        ensure_updated(affected, "task item does not exist")
    }

    pub fn items_for_task(&self, task_id: &str) -> Result<Vec<TaskItemRecord>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT
                item_id, task_id, pixiv_id, page_index, status, image_id,
                error_code, error_message, created_at, updated_at
             FROM task_items
             WHERE task_id = ?1
             ORDER BY created_at, item_id",
        )?;
        let items = stmt
            .query_map(params![task_id], row_to_task_item)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    pub fn add_log(&self, log: &NewTaskLogRecord) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO task_logs (
                log_id, task_id, level, phase, message, context_json, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, datetime('now')
             )",
            params![
                log.log_id,
                log.task_id,
                log.level.as_str(),
                log.phase,
                log.message,
                log.context_json
            ],
        )?;
        Ok(())
    }

    pub fn logs_for_task(&self, task_id: &str) -> Result<Vec<TaskLogRecord>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT log_id, task_id, level, phase, message, context_json, created_at
             FROM task_logs
             WHERE task_id = ?1
             ORDER BY created_at, log_id",
        )?;
        let logs = stmt
            .query_map(params![task_id], row_to_task_log)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(logs)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleDownloadTaskOutcome {
    pub task_id: String,
    pub image_id: Option<String>,
    pub download_status: DownloadItemStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorDownloadRequest {
    pub author_uid: String,
    pub limit: u32,
    pub r18_policy: R18Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkDownloadRequest {
    pub limit: u32,
    pub r18_policy: R18Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartDownloadRequest {
    pub prompt: String,
    pub tags: Vec<String>,
    pub negative_tags: Vec<String>,
    pub limit: u32,
    pub r18_policy: R18Policy,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchDownloadTaskOutcome {
    pub task_id: String,
    pub total: u32,
    pub completed: u32,
    pub failed: u32,
    pub status: TaskStatus,
}

pub fn run_single_download_task(
    request: &DownloadRequest,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<SingleDownloadTaskOutcome, AppError> {
    let task_id = create_single_download_task(request, conn)?;
    execute_single_download_task(&task_id, pixiv, storage, conn)
}

pub fn run_single_download_task_with_id(
    task_id: &str,
    request: &DownloadRequest,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<SingleDownloadTaskOutcome, AppError> {
    create_single_download_task_with_id(task_id, request, conn)?;
    execute_single_download_task(task_id, pixiv, storage, conn)
}

pub fn create_single_download_task(
    request: &DownloadRequest,
    conn: &Connection,
) -> Result<String, AppError> {
    let task_id = new_local_id("task");
    create_single_download_task_with_id(&task_id, request, conn)?;
    Ok(task_id)
}

pub fn create_single_download_task_with_id(
    task_id: &str,
    request: &DownloadRequest,
    conn: &Connection,
) -> Result<(), AppError> {
    let tasks = TaskRepository::new(conn);
    let item_id = format!("{task_id}:item:0");
    let page_index = request.page_index.unwrap_or(0);

    tasks.insert_task(&NewTaskRecord {
        task_id: task_id.to_owned(),
        task_type: TaskType::Single,
        request_json: single_request_json(request),
        progress_total: Some(1),
    })?;
    tasks.insert_item(&NewTaskItemRecord {
        item_id: item_id.clone(),
        task_id: task_id.to_owned(),
        pixiv_id: Some(request.pixiv_id.clone()),
        page_index: Some(page_index),
        status: TaskItemStatus::Discovered,
    })?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "validate_request",
        "Task created",
    )?;
    Ok(())
}

pub fn create_author_download_task(
    request: &AuthorDownloadRequest,
    conn: &Connection,
) -> Result<String, AppError> {
    let task_id = new_local_id("task");
    create_author_download_task_with_id(&task_id, request, conn)?;
    Ok(task_id)
}

pub fn create_author_download_task_with_id(
    task_id: &str,
    request: &AuthorDownloadRequest,
    conn: &Connection,
) -> Result<(), AppError> {
    validate_author_request(request)?;
    let tasks = TaskRepository::new(conn);
    tasks.insert_task(&NewTaskRecord {
        task_id: task_id.to_owned(),
        task_type: TaskType::Author,
        request_json: author_request_json(request),
        progress_total: Some(0),
    })?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "validate_request",
        "Author batch task created",
    )?;
    Ok(())
}

pub fn create_bookmark_download_task(
    request: &BookmarkDownloadRequest,
    conn: &Connection,
) -> Result<String, AppError> {
    let task_id = new_local_id("task");
    create_bookmark_download_task_with_id(&task_id, request, conn)?;
    Ok(task_id)
}

pub fn create_bookmark_download_task_with_id(
    task_id: &str,
    request: &BookmarkDownloadRequest,
    conn: &Connection,
) -> Result<(), AppError> {
    validate_bookmark_request(request)?;
    let tasks = TaskRepository::new(conn);
    tasks.insert_task(&NewTaskRecord {
        task_id: task_id.to_owned(),
        task_type: TaskType::Bookmark,
        request_json: bookmark_request_json(request),
        progress_total: Some(0),
    })?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "validate_request",
        "Bookmark batch task created",
    )?;
    Ok(())
}

pub fn create_smart_download_task(
    request: &SmartDownloadRequest,
    conn: &Connection,
) -> Result<String, AppError> {
    let task_id = new_local_id("task");
    create_smart_download_task_with_id(&task_id, request, conn)?;
    Ok(task_id)
}

pub fn create_smart_download_task_with_id(
    task_id: &str,
    request: &SmartDownloadRequest,
    conn: &Connection,
) -> Result<(), AppError> {
    validate_smart_request(request)?;
    let tasks = TaskRepository::new(conn);
    let request_json = smart_request_json(request);
    tasks.insert_task(&NewTaskRecord {
        task_id: task_id.to_owned(),
        task_type: TaskType::Smart,
        request_json: request_json.clone(),
        progress_total: Some(0),
    })?;
    conn.execute(
        "INSERT INTO smart_retrievals (
            retrieval_id, task_id, user_prompt, llm_model, llm_output_json,
            tags_json, negative_tags_json, requested_count, r18_policy, created_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now')
         )",
        params![
            new_local_id("smart"),
            task_id,
            request.prompt,
            request.model,
            request_json,
            serde_json::to_string(&request.tags).unwrap_or_else(|_| "[]".to_owned()),
            serde_json::to_string(&request.negative_tags).unwrap_or_else(|_| "[]".to_owned()),
            request.limit,
            request.r18_policy.as_str()
        ],
    )?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "validate_request",
        "Smart batch task created",
    )?;
    Ok(())
}

pub fn execute_single_download_task(
    task_id: &str,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<SingleDownloadTaskOutcome, AppError> {
    let tasks = TaskRepository::new(conn);
    let task = tasks
        .find_task(task_id)?
        .ok_or_else(|| AppError::new(ErrorCode::TaskNotFound, "task not found"))?;
    if task.task_type != TaskType::Single {
        return Err(AppError::validation("task is not a single download task"));
    }
    let request = parse_single_request_json(&task.request_json)?;
    let item_id = format!("{task_id}:item:0");
    let page_index = request.page_index.unwrap_or(0);
    let current_item = format!("{}:{page_index}", request.pixiv_id);

    tasks.start_task(task_id, Some(&current_item))?;
    tasks.update_item_status(&item_id, TaskItemStatus::Downloading, None, None, None)?;
    tasks.update_progress(task_id, 0, 0, Some(&current_item))?;

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "fetch_metadata",
        "Fetching Pixiv metadata when needed",
    )?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "deduplicate",
        "Checking DB and local file state",
    )?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "download_file",
        "Downloading image bytes only if required",
    )?;

    let download_context = DownloadRepositoryContext::for_task(conn, task_id);
    let outcome = match download_single_with_db(&request, pixiv, storage, &download_context) {
        Ok(outcome) => outcome,
        Err(error) => {
            tasks.update_item_status(
                &item_id,
                TaskItemStatus::ItemFailed,
                None,
                Some(error.code.as_str()),
                Some(&error.message),
            )?;
            tasks.update_progress(task_id, 0, 1, Some(&current_item))?;
            log_phase(
                &tasks,
                task_id,
                TaskLogLevel::Error,
                "finish_task",
                &error.message,
            )?;
            tasks.fail_task(task_id, error.code.as_str(), &error.message)?;
            return Err(error);
        }
    };

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "write_file",
        "File state is ready",
    )?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "index_image",
        "Image metadata and provenance are indexed",
    )?;

    let image_id = ImageRepository::new(conn)
        .find_by_pixiv_page(&outcome.pixiv_id, outcome.page_index)?
        .map(|image| image.image_id);
    let item_status = match outcome.status {
        DownloadItemStatus::Saved => TaskItemStatus::Saved,
        DownloadItemStatus::SkippedDuplicate => TaskItemStatus::DuplicateSkipped,
        DownloadItemStatus::SkippedByPolicy => TaskItemStatus::PolicySkipped,
        DownloadItemStatus::Failed => TaskItemStatus::ItemFailed,
    };
    tasks.update_item_status(&item_id, item_status, image_id.as_deref(), None, None)?;
    tasks.update_progress(task_id, 1, 0, Some(&current_item))?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "finish_task",
        "Task completed",
    )?;
    tasks.complete_task(task_id)?;

    Ok(SingleDownloadTaskOutcome {
        task_id: task_id.to_owned(),
        image_id,
        download_status: outcome.status,
    })
}

pub fn execute_author_download_task(
    task_id: &str,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<BatchDownloadTaskOutcome, AppError> {
    let tasks = TaskRepository::new(conn);
    let task = tasks
        .find_task(task_id)?
        .ok_or_else(|| AppError::new(ErrorCode::TaskNotFound, "task not found"))?;
    if task.task_type != TaskType::Author {
        return Err(AppError::validation("task is not an author download task"));
    }
    let request = parse_author_request_json(&task.request_json)?;
    validate_author_request(&request)?;
    let current_discovery = format!("author:{}", request.author_uid);

    tasks.start_task(task_id, Some(&current_discovery))?;
    tasks.update_progress(task_id, 0, 0, Some(&current_discovery))?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "discover_author_works",
        "Fetching Pixiv works for author",
    )?;

    let works = match pixiv.fetch_author_works(&request.author_uid, request.limit) {
        Ok(works) => works,
        Err(error) => {
            log_phase(
                &tasks,
                task_id,
                TaskLogLevel::Error,
                "discover_author_works",
                &error.message,
            )?;
            tasks.fail_task(task_id, error.code.as_str(), &error.message)?;
            return Err(error);
        }
    };
    let total = u32::try_from(works.len())
        .map_err(|_| AppError::validation("author batch result is too large"))?;
    tasks.set_progress_total(task_id, total)?;

    if works.is_empty() {
        log_phase(
            &tasks,
            task_id,
            TaskLogLevel::Info,
            "finish_task",
            "Author batch completed with no discovered works",
        )?;
        tasks.complete_task(task_id)?;
        return Ok(BatchDownloadTaskOutcome {
            task_id: task_id.to_owned(),
            total,
            completed: 0,
            failed: 0,
            status: TaskStatus::Completed,
        });
    }

    for (index, work_ref) in works.iter().enumerate() {
        let item_id = format!("{task_id}:item:{index}");
        tasks.insert_item(&NewTaskItemRecord {
            item_id,
            task_id: task_id.to_owned(),
            pixiv_id: Some(work_ref.pixiv_id.clone()),
            page_index: Some(0),
            status: TaskItemStatus::Discovered,
        })?;
    }

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "download_file",
        "Processing author works sequentially",
    )?;

    let mut completed = 0;
    let mut failed = 0;
    for (index, work_ref) in works.into_iter().enumerate() {
        let item_id = format!("{task_id}:item:{index}");
        let current_item = format!("{}:0", work_ref.pixiv_id);
        tasks.update_item_status(&item_id, TaskItemStatus::Downloading, None, None, None)?;
        tasks.update_progress(task_id, completed, failed, Some(&current_item))?;

        let download_request = DownloadRequest {
            pixiv_id: work_ref.pixiv_id.clone(),
            page_index: Some(0),
            source: ImageSource::Author,
            r18_policy: request.r18_policy,
        };
        let download_context = DownloadRepositoryContext::for_task(conn, task_id);
        match download_single_with_db(&download_request, pixiv, storage, &download_context) {
            Ok(outcome) => {
                let image_id = ImageRepository::new(conn)
                    .find_by_pixiv_page(&outcome.pixiv_id, outcome.page_index)?
                    .map(|image| image.image_id);
                let item_status = match outcome.status {
                    DownloadItemStatus::Saved => TaskItemStatus::Saved,
                    DownloadItemStatus::SkippedDuplicate => TaskItemStatus::DuplicateSkipped,
                    DownloadItemStatus::SkippedByPolicy => TaskItemStatus::PolicySkipped,
                    DownloadItemStatus::Failed => TaskItemStatus::ItemFailed,
                };
                tasks.update_item_status(&item_id, item_status, image_id.as_deref(), None, None)?;
                completed += 1;
            }
            Err(error) => {
                tasks.update_item_status(
                    &item_id,
                    TaskItemStatus::ItemFailed,
                    None,
                    Some(error.code.as_str()),
                    Some(&error.message),
                )?;
                failed += 1;
                log_phase(
                    &tasks,
                    task_id,
                    TaskLogLevel::Error,
                    "download_file",
                    &error.message,
                )?;
            }
        }
        tasks.update_progress(task_id, completed, failed, Some(&current_item))?;
    }

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "index_image",
        "Author batch image metadata and provenance are indexed",
    )?;
    let status = if failed > 0 {
        TaskStatus::CompletedWithErrors
    } else {
        TaskStatus::Completed
    };
    log_phase(
        &tasks,
        task_id,
        if failed > 0 {
            TaskLogLevel::Warn
        } else {
            TaskLogLevel::Info
        },
        "finish_task",
        if failed > 0 {
            "Author batch completed with item errors"
        } else {
            "Author batch completed"
        },
    )?;
    tasks.finish_task(task_id, status, None, None)?;

    Ok(BatchDownloadTaskOutcome {
        task_id: task_id.to_owned(),
        total,
        completed,
        failed,
        status,
    })
}

pub fn execute_bookmark_download_task(
    task_id: &str,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<BatchDownloadTaskOutcome, AppError> {
    let tasks = TaskRepository::new(conn);
    let task = tasks
        .find_task(task_id)?
        .ok_or_else(|| AppError::new(ErrorCode::TaskNotFound, "task not found"))?;
    if task.task_type != TaskType::Bookmark {
        return Err(AppError::validation("task is not a bookmark download task"));
    }
    let request = parse_bookmark_request_json(&task.request_json)?;
    validate_bookmark_request(&request)?;

    tasks.start_task(task_id, Some("bookmarks"))?;
    tasks.update_progress(task_id, 0, 0, Some("bookmarks"))?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "discover_bookmarks",
        "Fetching Pixiv bookmarked works",
    )?;

    let works = match pixiv.fetch_bookmarks(request.limit, request.r18_policy) {
        Ok(works) => works,
        Err(error) => {
            log_phase(
                &tasks,
                task_id,
                TaskLogLevel::Error,
                "discover_bookmarks",
                &error.message,
            )?;
            tasks.fail_task(task_id, error.code.as_str(), &error.message)?;
            return Err(error);
        }
    };
    let total = u32::try_from(works.len())
        .map_err(|_| AppError::validation("bookmark batch result is too large"))?;
    tasks.set_progress_total(task_id, total)?;

    if works.is_empty() {
        log_phase(
            &tasks,
            task_id,
            TaskLogLevel::Info,
            "finish_task",
            "Bookmark batch completed with no discovered works",
        )?;
        tasks.complete_task(task_id)?;
        return Ok(BatchDownloadTaskOutcome {
            task_id: task_id.to_owned(),
            total,
            completed: 0,
            failed: 0,
            status: TaskStatus::Completed,
        });
    }

    for (index, work_ref) in works.iter().enumerate() {
        let item_id = format!("{task_id}:item:{index}");
        tasks.insert_item(&NewTaskItemRecord {
            item_id,
            task_id: task_id.to_owned(),
            pixiv_id: Some(work_ref.pixiv_id.clone()),
            page_index: Some(0),
            status: TaskItemStatus::Discovered,
        })?;
    }

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "download_file",
        "Processing bookmarked works sequentially",
    )?;

    let mut completed = 0;
    let mut failed = 0;
    for (index, work_ref) in works.into_iter().enumerate() {
        let item_id = format!("{task_id}:item:{index}");
        let current_item = format!("{}:0", work_ref.pixiv_id);
        tasks.update_item_status(&item_id, TaskItemStatus::Downloading, None, None, None)?;
        tasks.update_progress(task_id, completed, failed, Some(&current_item))?;

        let download_request = DownloadRequest {
            pixiv_id: work_ref.pixiv_id.clone(),
            page_index: Some(0),
            source: ImageSource::Bookmark,
            r18_policy: request.r18_policy,
        };
        let download_context = DownloadRepositoryContext::for_task(conn, task_id);
        match download_single_with_db(&download_request, pixiv, storage, &download_context) {
            Ok(outcome) => {
                let image_id = ImageRepository::new(conn)
                    .find_by_pixiv_page(&outcome.pixiv_id, outcome.page_index)?
                    .map(|image| image.image_id);
                let item_status = match outcome.status {
                    DownloadItemStatus::Saved => TaskItemStatus::Saved,
                    DownloadItemStatus::SkippedDuplicate => TaskItemStatus::DuplicateSkipped,
                    DownloadItemStatus::SkippedByPolicy => TaskItemStatus::PolicySkipped,
                    DownloadItemStatus::Failed => TaskItemStatus::ItemFailed,
                };
                tasks.update_item_status(&item_id, item_status, image_id.as_deref(), None, None)?;
                completed += 1;
            }
            Err(error) => {
                tasks.update_item_status(
                    &item_id,
                    TaskItemStatus::ItemFailed,
                    None,
                    Some(error.code.as_str()),
                    Some(&error.message),
                )?;
                failed += 1;
                log_phase(
                    &tasks,
                    task_id,
                    TaskLogLevel::Error,
                    "download_file",
                    &error.message,
                )?;
            }
        }
        tasks.update_progress(task_id, completed, failed, Some(&current_item))?;
    }

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "index_image",
        "Bookmark batch image metadata and provenance are indexed",
    )?;
    let status = if failed > 0 {
        TaskStatus::CompletedWithErrors
    } else {
        TaskStatus::Completed
    };
    log_phase(
        &tasks,
        task_id,
        if failed > 0 {
            TaskLogLevel::Warn
        } else {
            TaskLogLevel::Info
        },
        "finish_task",
        if failed > 0 {
            "Bookmark batch completed with item errors"
        } else {
            "Bookmark batch completed"
        },
    )?;
    tasks.finish_task(task_id, status, None, None)?;

    Ok(BatchDownloadTaskOutcome {
        task_id: task_id.to_owned(),
        total,
        completed,
        failed,
        status,
    })
}

pub fn execute_smart_download_task(
    task_id: &str,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<BatchDownloadTaskOutcome, AppError> {
    let tasks = TaskRepository::new(conn);
    let task = tasks
        .find_task(task_id)?
        .ok_or_else(|| AppError::new(ErrorCode::TaskNotFound, "task not found"))?;
    if task.task_type != TaskType::Smart {
        return Err(AppError::validation("task is not a smart download task"));
    }
    let request = parse_smart_request_json(&task.request_json)?;
    validate_smart_request(&request)?;
    let current_discovery = format!("smart:{}", request.tags.join(" "));

    tasks.start_task(task_id, Some(&current_discovery))?;
    tasks.update_progress(task_id, 0, 0, Some(&current_discovery))?;
    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "discover_smart_works",
        "Searching Pixiv works by smart tags",
    )?;

    let works = match pixiv.search_works_by_tags(
        &request.tags,
        &request.negative_tags,
        request.limit,
        request.r18_policy,
    ) {
        Ok(works) => works,
        Err(error) => {
            log_phase(
                &tasks,
                task_id,
                TaskLogLevel::Error,
                "discover_smart_works",
                &error.message,
            )?;
            tasks.fail_task(task_id, error.code.as_str(), &error.message)?;
            return Err(error);
        }
    };
    let total = u32::try_from(works.len())
        .map_err(|_| AppError::validation("smart batch result is too large"))?;
    tasks.set_progress_total(task_id, total)?;

    if works.is_empty() {
        log_phase(
            &tasks,
            task_id,
            TaskLogLevel::Info,
            "finish_task",
            "Smart batch completed with no discovered works",
        )?;
        tasks.complete_task(task_id)?;
        return Ok(BatchDownloadTaskOutcome {
            task_id: task_id.to_owned(),
            total,
            completed: 0,
            failed: 0,
            status: TaskStatus::Completed,
        });
    }

    for (index, work_ref) in works.iter().enumerate() {
        let item_id = format!("{task_id}:item:{index}");
        tasks.insert_item(&NewTaskItemRecord {
            item_id,
            task_id: task_id.to_owned(),
            pixiv_id: Some(work_ref.pixiv_id.clone()),
            page_index: Some(0),
            status: TaskItemStatus::Discovered,
        })?;
    }

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "download_file",
        "Processing smart search works sequentially",
    )?;

    let mut completed = 0;
    let mut failed = 0;
    for (index, work_ref) in works.into_iter().enumerate() {
        let item_id = format!("{task_id}:item:{index}");
        let current_item = format!("{}:0", work_ref.pixiv_id);
        tasks.update_item_status(&item_id, TaskItemStatus::Downloading, None, None, None)?;
        tasks.update_progress(task_id, completed, failed, Some(&current_item))?;

        let download_request = DownloadRequest {
            pixiv_id: work_ref.pixiv_id.clone(),
            page_index: Some(0),
            source: ImageSource::Smart,
            r18_policy: request.r18_policy,
        };
        let download_context = DownloadRepositoryContext::for_task(conn, task_id);
        match download_single_with_db(&download_request, pixiv, storage, &download_context) {
            Ok(outcome) => {
                let image_id = ImageRepository::new(conn)
                    .find_by_pixiv_page(&outcome.pixiv_id, outcome.page_index)?
                    .map(|image| image.image_id);
                let item_status = match outcome.status {
                    DownloadItemStatus::Saved => TaskItemStatus::Saved,
                    DownloadItemStatus::SkippedDuplicate => TaskItemStatus::DuplicateSkipped,
                    DownloadItemStatus::SkippedByPolicy => TaskItemStatus::PolicySkipped,
                    DownloadItemStatus::Failed => TaskItemStatus::ItemFailed,
                };
                tasks.update_item_status(&item_id, item_status, image_id.as_deref(), None, None)?;
                completed += 1;
            }
            Err(error) => {
                tasks.update_item_status(
                    &item_id,
                    TaskItemStatus::ItemFailed,
                    None,
                    Some(error.code.as_str()),
                    Some(&error.message),
                )?;
                failed += 1;
                log_phase(
                    &tasks,
                    task_id,
                    TaskLogLevel::Error,
                    "download_file",
                    &error.message,
                )?;
            }
        }
        tasks.update_progress(task_id, completed, failed, Some(&current_item))?;
    }

    log_phase(
        &tasks,
        task_id,
        TaskLogLevel::Info,
        "index_image",
        "Smart batch image metadata and provenance are indexed",
    )?;
    let status = if failed > 0 {
        TaskStatus::CompletedWithErrors
    } else {
        TaskStatus::Completed
    };
    log_phase(
        &tasks,
        task_id,
        if failed > 0 {
            TaskLogLevel::Warn
        } else {
            TaskLogLevel::Info
        },
        "finish_task",
        if failed > 0 {
            "Smart batch completed with item errors"
        } else {
            "Smart batch completed"
        },
    )?;
    tasks.finish_task(task_id, status, None, None)?;

    Ok(BatchDownloadTaskOutcome {
        task_id: task_id.to_owned(),
        total,
        completed,
        failed,
        status,
    })
}

pub fn execute_queued_task(
    task_id: &str,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    conn: &Connection,
) -> Result<(), AppError> {
    let task = TaskRepository::new(conn)
        .find_task(task_id)?
        .ok_or_else(|| AppError::new(ErrorCode::TaskNotFound, "task not found"))?;
    match task.task_type {
        TaskType::Single => {
            execute_single_download_task(task_id, pixiv, storage, conn)?;
            Ok(())
        }
        TaskType::Author => {
            execute_author_download_task(task_id, pixiv, storage, conn)?;
            Ok(())
        }
        TaskType::Bookmark => {
            execute_bookmark_download_task(task_id, pixiv, storage, conn)?;
            Ok(())
        }
        TaskType::Smart => {
            execute_smart_download_task(task_id, pixiv, storage, conn)?;
            Ok(())
        }
        _ => Err(AppError::validation(
            "task type is not implemented by worker",
        )),
    }
}

pub fn single_request_json(request: &DownloadRequest) -> String {
    serde_json::json!({
        "pixiv_id": request.pixiv_id,
        "page_index": request.page_index,
        "source": request.source.as_str(),
        "r18_policy": request.r18_policy.as_str(),
    })
    .to_string()
}

pub fn author_request_json(request: &AuthorDownloadRequest) -> String {
    serde_json::json!({
        "author_uid": request.author_uid,
        "limit": request.limit,
        "source": ImageSource::Author.as_str(),
        "r18_policy": request.r18_policy.as_str(),
    })
    .to_string()
}

pub fn bookmark_request_json(request: &BookmarkDownloadRequest) -> String {
    serde_json::json!({
        "limit": request.limit,
        "source": ImageSource::Bookmark.as_str(),
        "r18_policy": request.r18_policy.as_str(),
    })
    .to_string()
}

pub fn smart_request_json(request: &SmartDownloadRequest) -> String {
    serde_json::json!({
        "prompt": request.prompt,
        "tags": request.tags,
        "negative_tags": request.negative_tags,
        "limit": request.limit,
        "source": ImageSource::Smart.as_str(),
        "r18_policy": request.r18_policy.as_str(),
        "model": request.model,
    })
    .to_string()
}

fn validate_author_request(request: &AuthorDownloadRequest) -> Result<(), AppError> {
    if request.author_uid.trim().is_empty() {
        return Err(AppError::validation("author_uid cannot be empty"));
    }
    if !request.author_uid.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::validation("author_uid must contain only digits"));
    }
    if request.limit == 0 {
        return Err(AppError::validation("limit must be at least 1"));
    }
    Ok(())
}

fn validate_bookmark_request(request: &BookmarkDownloadRequest) -> Result<(), AppError> {
    if request.limit == 0 {
        return Err(AppError::validation("limit must be at least 1"));
    }
    Ok(())
}

fn validate_smart_request(request: &SmartDownloadRequest) -> Result<(), AppError> {
    if request.prompt.trim().is_empty() {
        return Err(AppError::validation("prompt cannot be empty"));
    }
    if request.tags.iter().all(|tag| tag.trim().is_empty()) {
        return Err(AppError::validation("tags cannot be empty"));
    }
    if request.limit == 0 {
        return Err(AppError::validation("limit must be at least 1"));
    }
    Ok(())
}

fn parse_single_request_json(raw: &str) -> Result<DownloadRequest, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        AppError::new(
            ErrorCode::ValidationError,
            format!("task request_json is invalid: {error}"),
        )
    })?;
    let pixiv_id = value
        .get("pixiv_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::validation("task request_json missing pixiv_id"))?
        .to_owned();
    let page_index = match value.get("page_index") {
        Some(Value::Null) | None => None,
        Some(Value::Number(number)) => {
            let raw = number
                .as_u64()
                .ok_or_else(|| AppError::validation("task request_json page_index is invalid"))?;
            Some(
                u32::try_from(raw)
                    .map_err(|_| AppError::validation("task request_json page_index is invalid"))?,
            )
        }
        _ => {
            return Err(AppError::validation(
                "task request_json page_index is invalid",
            ));
        }
    };
    let source = value
        .get("source")
        .and_then(Value::as_str)
        .and_then(ImageSource::from_db)
        .ok_or_else(|| AppError::validation("task request_json source is invalid"))?;
    let r18_policy = value
        .get("r18_policy")
        .and_then(Value::as_str)
        .and_then(R18Policy::from_api)
        .ok_or_else(|| AppError::validation("task request_json r18_policy is invalid"))?;

    Ok(DownloadRequest {
        pixiv_id,
        page_index,
        source,
        r18_policy,
    })
}

fn parse_author_request_json(raw: &str) -> Result<AuthorDownloadRequest, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        AppError::new(
            ErrorCode::ValidationError,
            format!("task request_json is invalid: {error}"),
        )
    })?;
    let author_uid = value
        .get("author_uid")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::validation("task request_json missing author_uid"))?
        .to_owned();
    let limit = value
        .get("limit")
        .and_then(Value::as_u64)
        .ok_or_else(|| AppError::validation("task request_json limit is invalid"))
        .and_then(|raw| {
            u32::try_from(raw)
                .map_err(|_| AppError::validation("task request_json limit is invalid"))
        })?;
    let r18_policy = value
        .get("r18_policy")
        .and_then(Value::as_str)
        .and_then(R18Policy::from_api)
        .ok_or_else(|| AppError::validation("task request_json r18_policy is invalid"))?;

    Ok(AuthorDownloadRequest {
        author_uid,
        limit,
        r18_policy,
    })
}

fn parse_bookmark_request_json(raw: &str) -> Result<BookmarkDownloadRequest, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        AppError::new(
            ErrorCode::ValidationError,
            format!("task request_json is invalid: {error}"),
        )
    })?;
    let limit = value
        .get("limit")
        .and_then(Value::as_u64)
        .ok_or_else(|| AppError::validation("task request_json limit is invalid"))
        .and_then(|raw| {
            u32::try_from(raw)
                .map_err(|_| AppError::validation("task request_json limit is invalid"))
        })?;
    let r18_policy = value
        .get("r18_policy")
        .and_then(Value::as_str)
        .and_then(R18Policy::from_api)
        .ok_or_else(|| AppError::validation("task request_json r18_policy is invalid"))?;

    Ok(BookmarkDownloadRequest { limit, r18_policy })
}

fn parse_smart_request_json(raw: &str) -> Result<SmartDownloadRequest, AppError> {
    let value: Value = serde_json::from_str(raw).map_err(|error| {
        AppError::new(
            ErrorCode::ValidationError,
            format!("task request_json is invalid: {error}"),
        )
    })?;
    let prompt = value
        .get("prompt")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::validation("task request_json missing prompt"))?
        .to_owned();
    let tags = value_to_string_vec(
        value
            .get("tags")
            .ok_or_else(|| AppError::validation("task request_json missing tags"))?,
    )?;
    let negative_tags = value
        .get("negative_tags")
        .map(value_to_string_vec)
        .transpose()?
        .unwrap_or_default();
    let limit = value
        .get("limit")
        .and_then(Value::as_u64)
        .ok_or_else(|| AppError::validation("task request_json limit is invalid"))
        .and_then(|raw| {
            u32::try_from(raw)
                .map_err(|_| AppError::validation("task request_json limit is invalid"))
        })?;
    let r18_policy = value
        .get("r18_policy")
        .and_then(Value::as_str)
        .and_then(R18Policy::from_api)
        .ok_or_else(|| AppError::validation("task request_json r18_policy is invalid"))?;
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("manual-tags")
        .to_owned();

    Ok(SmartDownloadRequest {
        prompt,
        tags,
        negative_tags,
        limit,
        r18_policy,
        model,
    })
}

fn value_to_string_vec(value: &Value) -> Result<Vec<String>, AppError> {
    value
        .as_array()
        .ok_or_else(|| AppError::validation("task request_json tags are invalid"))?
        .iter()
        .map(|tag| {
            tag.as_str()
                .map(str::to_owned)
                .ok_or_else(|| AppError::validation("task request_json tags are invalid"))
        })
        .collect()
}

fn log_phase(
    tasks: &TaskRepository<'_>,
    task_id: &str,
    level: TaskLogLevel,
    phase: &str,
    message: &str,
) -> Result<(), AppError> {
    tasks.add_log(&NewTaskLogRecord {
        log_id: new_local_id("log"),
        task_id: task_id.to_owned(),
        level,
        phase: phase.to_owned(),
        message: message.to_owned(),
        context_json: None,
    })
}

fn new_local_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{prefix}-{}-{nanos}", std::process::id())
}

fn ensure_updated(affected: usize, message: &str) -> Result<(), AppError> {
    if affected == 0 {
        return Err(AppError::validation(message));
    }
    Ok(())
}

fn row_to_task(row: &rusqlite::Row<'_>) -> Result<TaskRecord, rusqlite::Error> {
    let task_type_value: String = row.get(1)?;
    let task_type = TaskType::from_db(&task_type_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            format!("invalid task type: {task_type_value}").into(),
        )
    })?;
    let status_value: String = row.get(2)?;
    let status = TaskStatus::from_db(&status_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            format!("invalid task status: {status_value}").into(),
        )
    })?;

    Ok(TaskRecord {
        task_id: row.get(0)?,
        task_type,
        status,
        request_json: row.get(3)?,
        progress_total: row.get(4)?,
        progress_done: row.get(5)?,
        progress_failed: row.get(6)?,
        current_item: row.get(7)?,
        error_code: row.get(8)?,
        error_message: row.get(9)?,
        created_at: row.get(10)?,
        started_at: row.get(11)?,
        finished_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn row_to_task_log(row: &rusqlite::Row<'_>) -> Result<TaskLogRecord, rusqlite::Error> {
    let level_value: String = row.get(2)?;
    let level = TaskLogLevel::from_db(&level_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            format!("invalid task log level: {level_value}").into(),
        )
    })?;

    Ok(TaskLogRecord {
        log_id: row.get(0)?,
        task_id: row.get(1)?,
        level,
        phase: row.get(3)?,
        message: row.get(4)?,
        context_json: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn row_to_task_item(row: &rusqlite::Row<'_>) -> Result<TaskItemRecord, rusqlite::Error> {
    let status_value: String = row.get(4)?;
    let status = TaskItemStatus::from_db(&status_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Text,
            format!("invalid task item status: {status_value}").into(),
        )
    })?;

    Ok(TaskItemRecord {
        item_id: row.get(0)?,
        task_id: row.get(1)?,
        pixiv_id: row.get(2)?,
        page_index: row.get(3)?,
        status,
        image_id: row.get(5)?,
        error_code: row.get(6)?,
        error_message: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::db::open_in_memory;
    use crate::domain::{
        DownloadItemStatus, DownloadRequest, ImageCategory, ImageSource, PixivPage, PixivWork,
        PixivWorkRef, R18Policy, TaskItemStatus, TaskLogLevel, TaskStatus, TaskType,
    };
    use crate::images::ImageRepository;
    use crate::pixiv::mock::MockPixivClient;
    use crate::storage::StoragePlanner;
    use crate::tasks::{
        AuthorDownloadRequest, BookmarkDownloadRequest, NewTaskItemRecord, NewTaskLogRecord,
        NewTaskRecord, SmartDownloadRequest, TaskListQuery, TaskRepository,
        create_author_download_task_with_id, create_bookmark_download_task_with_id,
        create_smart_download_task_with_id, execute_author_download_task,
        execute_bookmark_download_task, execute_smart_download_task,
        run_single_download_task_with_id,
    };

    fn sample_work() -> PixivWork {
        sample_work_with_id("123456")
    }

    fn sample_work_with_id(pixiv_id: &str) -> PixivWork {
        PixivWork {
            pixiv_id: pixiv_id.to_owned(),
            title: Some(format!("task mock {pixiv_id}")),
            author_uid: Some("9988".to_owned()),
            author_name: Some("mock author".to_owned()),
            tags: vec!["blue hair".to_owned()],
            category: ImageCategory::Normal,
            pages: vec![PixivPage {
                page_index: 0,
                original_url: format!("https://i.pximg.net/img-original/mock/{pixiv_id}_p0.jpg"),
                width: Some(1200),
                height: Some(1800),
                extension: Some("jpg".to_owned()),
            }],
        }
    }

    fn request() -> DownloadRequest {
        DownloadRequest {
            pixiv_id: "123456".to_owned(),
            page_index: Some(0),
            source: ImageSource::Single,
            r18_policy: R18Policy::Exclude,
        }
    }

    fn author_request(limit: u32) -> AuthorDownloadRequest {
        AuthorDownloadRequest {
            author_uid: "9988".to_owned(),
            limit,
            r18_policy: R18Policy::Exclude,
        }
    }

    fn bookmark_request(limit: u32) -> BookmarkDownloadRequest {
        BookmarkDownloadRequest {
            limit,
            r18_policy: R18Policy::Exclude,
        }
    }

    fn smart_request(limit: u32) -> SmartDownloadRequest {
        SmartDownloadRequest {
            prompt: "blue cyberpunk girl".to_owned(),
            tags: vec!["blue hair".to_owned(), "cyberpunk".to_owned()],
            negative_tags: vec!["low quality".to_owned()],
            limit,
            r18_policy: R18Policy::Exclude,
            model: "deepseek-v4-flash".to_owned(),
        }
    }

    fn test_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_backend_tasks_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn req_task_002_repository_persists_task_items_and_logs() {
        let conn = open_in_memory().unwrap();
        let repo = TaskRepository::new(&conn);

        repo.insert_task(&NewTaskRecord {
            task_id: "task-1".to_owned(),
            task_type: TaskType::Single,
            request_json: "{}".to_owned(),
            progress_total: Some(1),
        })
        .unwrap();
        repo.insert_item(&NewTaskItemRecord {
            item_id: "item-1".to_owned(),
            task_id: "task-1".to_owned(),
            pixiv_id: Some("123456".to_owned()),
            page_index: Some(0),
            status: TaskItemStatus::Discovered,
        })
        .unwrap();
        repo.add_log(&NewTaskLogRecord {
            log_id: "log-1".to_owned(),
            task_id: "task-1".to_owned(),
            level: TaskLogLevel::Info,
            phase: "validate_request".to_owned(),
            message: "created".to_owned(),
            context_json: None,
        })
        .unwrap();

        let task = repo.find_task("task-1").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.progress_total, Some(1));
        assert_eq!(repo.items_for_task("task-1").unwrap().len(), 1);
        assert_eq!(
            repo.logs_for_task("task-1").unwrap()[0].phase,
            "validate_request"
        );
    }

    #[test]
    fn req_task_002_repository_lists_tasks_with_filters_and_cursor() {
        let conn = open_in_memory().unwrap();
        let repo = TaskRepository::new(&conn);
        repo.insert_task(&NewTaskRecord {
            task_id: "task-1".to_owned(),
            task_type: TaskType::Single,
            request_json: "{}".to_owned(),
            progress_total: Some(1),
        })
        .unwrap();
        repo.insert_task(&NewTaskRecord {
            task_id: "task-2".to_owned(),
            task_type: TaskType::Smart,
            request_json: "{}".to_owned(),
            progress_total: Some(3),
        })
        .unwrap();
        repo.start_task("task-2", Some("smart")).unwrap();

        let running = repo
            .list_tasks(&TaskListQuery {
                status: Some(TaskStatus::Running),
                task_type: None,
                limit: 10,
                cursor_offset: 0,
            })
            .unwrap();
        assert_eq!(running.items.len(), 1);
        assert_eq!(running.items[0].task_id, "task-2");

        let first = repo
            .list_tasks(&TaskListQuery {
                status: None,
                task_type: None,
                limit: 1,
                cursor_offset: 0,
            })
            .unwrap();
        assert_eq!(first.items.len(), 1);
        assert_eq!(first.next_cursor_offset, Some(1));
    }

    #[test]
    fn req_task_003_req_task_005_enforces_explicit_and_monotonic_task_transitions() {
        let conn = open_in_memory().unwrap();
        let repo = TaskRepository::new(&conn);
        repo.insert_task(&NewTaskRecord {
            task_id: "task-1".to_owned(),
            task_type: TaskType::Single,
            request_json: "{}".to_owned(),
            progress_total: Some(2),
        })
        .unwrap();

        repo.start_task("task-1", Some("123456:0")).unwrap();
        repo.update_progress("task-1", 1, 0, Some("123456:0"))
            .unwrap();
        let regression = repo.update_progress("task-1", 0, 0, Some("123456:0"));
        assert!(regression.is_err());

        repo.complete_task("task-1").unwrap();
        let task = repo.find_task("task-1").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.started_at.is_some());
        assert!(task.finished_at.is_some());
        assert!(repo.start_task("task-1", None).is_err());
    }

    #[test]
    fn req_task_001_single_download_task_completes_and_links_image() {
        let root = test_root("single_success");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default()
            .with_work(sample_work())
            .with_image(
                "https://i.pximg.net/img-original/mock/123456_p0.jpg",
                b"fake image bytes".to_vec(),
            );
        let storage = StoragePlanner::new(&root);

        let outcome =
            run_single_download_task_with_id("task-single", &request(), &client, &storage, &conn)
                .unwrap();

        assert_eq!(outcome.download_status, DownloadItemStatus::Saved);
        assert!(outcome.image_id.is_some());
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-single").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.progress_done, 1);
        assert_eq!(task.progress_failed, 0);
        let item = repo.items_for_task("task-single").unwrap().pop().unwrap();
        assert_eq!(item.status, TaskItemStatus::Saved);
        assert_eq!(item.image_id, outcome.image_id);
        let image_repo = ImageRepository::new(&conn);
        let sources = image_repo
            .sources_for_image(outcome.image_id.as_deref().unwrap())
            .unwrap();
        assert_eq!(sources[0].task_id.as_deref(), Some("task-single"));
        let phases: Vec<_> = repo
            .logs_for_task("task-single")
            .unwrap()
            .into_iter()
            .map(|log| log.phase)
            .collect();
        assert!(phases.contains(&"validate_request".to_owned()));
        assert!(phases.contains(&"finish_task".to_owned()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_task_004_single_download_task_records_failure_diagnostics() {
        let root = test_root("single_failure");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default().with_work(sample_work());
        let storage = StoragePlanner::new(&root);

        let error =
            run_single_download_task_with_id("task-failed", &request(), &client, &storage, &conn)
                .unwrap_err();

        assert_eq!(error.code.as_str(), "PIXIV_NETWORK_ERROR");
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-failed").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.progress_done, 0);
        assert_eq!(task.progress_failed, 1);
        assert_eq!(task.error_code.as_deref(), Some("PIXIV_NETWORK_ERROR"));
        let item = repo.items_for_task("task-failed").unwrap().pop().unwrap();
        assert_eq!(item.status, TaskItemStatus::ItemFailed);
        assert_eq!(item.error_code.as_deref(), Some("PIXIV_NETWORK_ERROR"));
        assert!(
            repo.logs_for_task("task-failed")
                .unwrap()
                .iter()
                .any(|log| log.level == TaskLogLevel::Error)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_003_author_batch_task_completes_multiple_items() {
        let root = test_root("author_success");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default()
            .with_work(sample_work_with_id("123456"))
            .with_work(sample_work_with_id("222222"))
            .with_author_works(
                "9988",
                vec![
                    PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    },
                    PixivWorkRef {
                        pixiv_id: "222222".to_owned(),
                    },
                ],
            )
            .with_image(
                "https://i.pximg.net/img-original/mock/123456_p0.jpg",
                b"first image bytes".to_vec(),
            )
            .with_image(
                "https://i.pximg.net/img-original/mock/222222_p0.jpg",
                b"second image bytes".to_vec(),
            );
        let storage = StoragePlanner::new(&root);

        create_author_download_task_with_id("task-author", &author_request(2), &conn).unwrap();
        let outcome =
            execute_author_download_task("task-author", &client, &storage, &conn).unwrap();

        assert_eq!(outcome.status, TaskStatus::Completed);
        assert_eq!(outcome.total, 2);
        assert_eq!(outcome.completed, 2);
        assert_eq!(outcome.failed, 0);
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-author").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.progress_total, Some(2));
        assert_eq!(task.progress_done, 2);
        assert_eq!(task.progress_failed, 0);
        let items = repo.items_for_task("task-author").unwrap();
        assert_eq!(items.len(), 2);
        assert!(
            items
                .iter()
                .all(|item| item.status == TaskItemStatus::Saved)
        );
        let image_repo = ImageRepository::new(&conn);
        let image = image_repo.find_by_pixiv_page("222222", 0).unwrap().unwrap();
        let sources = image_repo.sources_for_image(&image.image_id).unwrap();
        assert!(
            sources
                .iter()
                .any(|source| source.source == ImageSource::Author
                    && source.task_id.as_deref() == Some("task-author"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_003_author_batch_task_preserves_item_failure_diagnostics() {
        let root = test_root("author_partial_failure");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default()
            .with_work(sample_work_with_id("123456"))
            .with_author_works(
                "9988",
                vec![
                    PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    },
                    PixivWorkRef {
                        pixiv_id: "404404".to_owned(),
                    },
                ],
            )
            .with_image(
                "https://i.pximg.net/img-original/mock/123456_p0.jpg",
                b"first image bytes".to_vec(),
            );
        let storage = StoragePlanner::new(&root);

        create_author_download_task_with_id("task-author-errors", &author_request(2), &conn)
            .unwrap();
        let outcome =
            execute_author_download_task("task-author-errors", &client, &storage, &conn).unwrap();

        assert_eq!(outcome.status, TaskStatus::CompletedWithErrors);
        assert_eq!(outcome.completed, 1);
        assert_eq!(outcome.failed, 1);
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-author-errors").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::CompletedWithErrors);
        assert_eq!(task.progress_done, 1);
        assert_eq!(task.progress_failed, 1);
        let items = repo.items_for_task("task-author-errors").unwrap();
        assert!(
            items
                .iter()
                .any(|item| item.status == TaskItemStatus::ItemFailed
                    && item.error_code.as_deref() == Some("PIXIV_NOT_FOUND"))
        );
        assert!(
            repo.logs_for_task("task-author-errors")
                .unwrap()
                .iter()
                .any(|log| log.level == TaskLogLevel::Error && log.phase == "download_file")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_002_bookmark_batch_task_completes_multiple_items() {
        let root = test_root("bookmark_success");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default()
            .with_work(sample_work_with_id("123456"))
            .with_work(sample_work_with_id("222222"))
            .with_bookmarks(vec![
                PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                },
                PixivWorkRef {
                    pixiv_id: "222222".to_owned(),
                },
            ])
            .with_image(
                "https://i.pximg.net/img-original/mock/123456_p0.jpg",
                b"first image bytes".to_vec(),
            )
            .with_image(
                "https://i.pximg.net/img-original/mock/222222_p0.jpg",
                b"second image bytes".to_vec(),
            );
        let storage = StoragePlanner::new(&root);

        create_bookmark_download_task_with_id("task-bookmark", &bookmark_request(2), &conn)
            .unwrap();
        let outcome =
            execute_bookmark_download_task("task-bookmark", &client, &storage, &conn).unwrap();

        assert_eq!(outcome.status, TaskStatus::Completed);
        assert_eq!(outcome.total, 2);
        assert_eq!(outcome.completed, 2);
        assert_eq!(outcome.failed, 0);
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-bookmark").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.progress_total, Some(2));
        assert_eq!(task.progress_done, 2);
        let items = repo.items_for_task("task-bookmark").unwrap();
        assert_eq!(items.len(), 2);
        assert!(
            items
                .iter()
                .all(|item| item.status == TaskItemStatus::Saved)
        );
        let image_repo = ImageRepository::new(&conn);
        let image = image_repo.find_by_pixiv_page("222222", 0).unwrap().unwrap();
        let sources = image_repo.sources_for_image(&image.image_id).unwrap();
        assert!(
            sources
                .iter()
                .any(|source| source.source == ImageSource::Bookmark
                    && source.task_id.as_deref() == Some("task-bookmark"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_002_bookmark_batch_task_preserves_item_failure_diagnostics() {
        let root = test_root("bookmark_partial_failure");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default()
            .with_work(sample_work_with_id("123456"))
            .with_bookmarks(vec![
                PixivWorkRef {
                    pixiv_id: "123456".to_owned(),
                },
                PixivWorkRef {
                    pixiv_id: "404404".to_owned(),
                },
            ])
            .with_image(
                "https://i.pximg.net/img-original/mock/123456_p0.jpg",
                b"first image bytes".to_vec(),
            );
        let storage = StoragePlanner::new(&root);

        create_bookmark_download_task_with_id("task-bookmark-errors", &bookmark_request(2), &conn)
            .unwrap();
        let outcome =
            execute_bookmark_download_task("task-bookmark-errors", &client, &storage, &conn)
                .unwrap();

        assert_eq!(outcome.status, TaskStatus::CompletedWithErrors);
        assert_eq!(outcome.completed, 1);
        assert_eq!(outcome.failed, 1);
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-bookmark-errors").unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::CompletedWithErrors);
        assert_eq!(task.progress_done, 1);
        assert_eq!(task.progress_failed, 1);
        let items = repo.items_for_task("task-bookmark-errors").unwrap();
        assert!(
            items
                .iter()
                .any(|item| item.status == TaskItemStatus::ItemFailed
                    && item.error_code.as_deref() == Some("PIXIV_NOT_FOUND"))
        );
        assert!(
            repo.logs_for_task("task-bookmark-errors")
                .unwrap()
                .iter()
                .any(|log| log.level == TaskLogLevel::Error && log.phase == "download_file")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_ai_002_smart_batch_task_completes_multiple_items_and_provenance() {
        let root = test_root("smart_success");
        let conn = open_in_memory().unwrap();
        let client = MockPixivClient::default()
            .with_work(sample_work_with_id("123456"))
            .with_work(sample_work_with_id("222222"))
            .with_tag_search(
                vec!["blue hair".to_owned(), "cyberpunk".to_owned()],
                vec![
                    PixivWorkRef {
                        pixiv_id: "123456".to_owned(),
                    },
                    PixivWorkRef {
                        pixiv_id: "222222".to_owned(),
                    },
                ],
            )
            .with_image(
                "https://i.pximg.net/img-original/mock/123456_p0.jpg",
                b"first image bytes".to_vec(),
            )
            .with_image(
                "https://i.pximg.net/img-original/mock/222222_p0.jpg",
                b"second image bytes".to_vec(),
            );
        let storage = StoragePlanner::new(&root);

        create_smart_download_task_with_id("task-smart", &smart_request(2), &conn).unwrap();
        let outcome = execute_smart_download_task("task-smart", &client, &storage, &conn).unwrap();

        assert_eq!(outcome.status, TaskStatus::Completed);
        assert_eq!(outcome.total, 2);
        assert_eq!(outcome.completed, 2);
        let repo = TaskRepository::new(&conn);
        let task = repo.find_task("task-smart").unwrap().unwrap();
        assert_eq!(task.task_type, TaskType::Smart);
        assert_eq!(task.progress_total, Some(2));
        assert_eq!(task.progress_done, 2);
        let items = repo.items_for_task("task-smart").unwrap();
        assert_eq!(items.len(), 2);
        assert!(
            items
                .iter()
                .all(|item| item.status == TaskItemStatus::Saved)
        );
        let image_repo = ImageRepository::new(&conn);
        let image = image_repo.find_by_pixiv_page("222222", 0).unwrap().unwrap();
        let sources = image_repo.sources_for_image(&image.image_id).unwrap();
        assert!(
            sources
                .iter()
                .any(|source| source.source == ImageSource::Smart
                    && source.task_id.as_deref() == Some("task-smart"))
        );
        let retrieval_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM smart_retrievals WHERE task_id = 'task-smart'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(retrieval_count, 1);
        let _ = fs::remove_dir_all(root);
    }
}
