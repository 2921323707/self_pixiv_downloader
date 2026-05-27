use axum::Json;
use axum::extract::{Path, Query, State};

use crate::api::AppState;
use crate::api::dto::{
    TaskListParams, TaskListResponse, TaskResponse, task_response, task_summary_response,
};
use crate::api::error::{ApiEnvelope, ApiError};
use crate::api::runtime::prepare_local_paths;
use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::tasks::TaskRepository;

pub(crate) async fn list_tasks(
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

pub(crate) async fn get_task(
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
