use std::fs;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};

use crate::api::AppState;
use crate::api::dto::{
    ImageDeleteBatchRequest, ImageDeleteBatchResponse, ImageDeleteItemResponse,
    ImageDetailResponse, ImageListParams, ImageListResponse, image_delete_error_response,
    image_delete_success_response, image_detail_response, image_summary_response,
    normalize_image_delete_ids,
};
use crate::api::error::{ApiEnvelope, ApiError};
use crate::api::runtime::{prepare_local_paths, resolve_download_root};
use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::images::{ImageRepository, delete_image_file_and_index, resolve_image_file};
use crate::settings::SettingsRepository;

pub(crate) async fn list_images(
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

pub(crate) async fn get_image(
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

pub(crate) async fn get_image_file(
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

pub(crate) async fn delete_image(
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

pub(crate) async fn post_delete_images(
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
