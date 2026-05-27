use std::fs;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::api::dto::{
    BatchDownloadResponse, SmartDownloadApiRequest, SmartParseApiRequest, SmartParseResponse,
    smart_parse_response,
};
use crate::api::error::{ApiEnvelope, ApiError};
use crate::api::runtime::{prepare_db_path, resolve_deepseek_config, resolve_runtime_settings};
use crate::api::worker::mark_task_failed;
use crate::api::{AppState, QueuedTask};
use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::settings::SettingsRepository;
use crate::tasks::create_smart_download_task;

pub(crate) async fn post_smart_parse(
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

pub(crate) async fn post_smart_download(
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
