use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};

use crate::accounts::PixivAccountRepository;
use crate::api::AppState;
use crate::api::dto::{
    DeepSeekConnectionTestResponse, PixivConnectionTestRequest, PixivConnectionTestResponse,
    SettingResponse, SettingUpdateRequest, SettingsListResponse, deepseek_connection_response,
    setting_response,
};
use crate::api::error::{ApiEnvelope, ApiError};
use crate::api::runtime::{
    prepare_db_path, prepare_local_paths, resolve_deepseek_config, resolve_runtime_settings,
};
use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::settings::SettingsRepository;

pub(crate) async fn get_settings(
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

pub(crate) async fn put_setting(
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

pub(crate) async fn post_test_pixiv(
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
        let profile = pixiv.fetch_current_user_profile().ok();
        if let (Some(cookie), Some(profile)) = (runtime.pixiv_cookie.as_deref(), profile.as_ref()) {
            let accounts = PixivAccountRepository::new(&conn);
            accounts.upsert_active(&profile.user_uid, profile.user_name.as_deref(), cookie)?;
            let settings = SettingsRepository::new(&conn);
            settings.upsert(
                "pixiv_active_account_uid",
                &serde_json::json!(profile.user_uid).to_string(),
                false,
            )?;
            settings.upsert(
                "pixiv_active_account_name",
                &serde_json::json!(
                    profile
                        .user_name
                        .clone()
                        .unwrap_or_else(|| format!("Pixiv UID {}", profile.user_uid))
                )
                .to_string(),
                false,
            )?;
        }
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
                user_uid: profile.as_ref().map(|profile| profile.user_uid.clone()),
                user_name: profile.and_then(|profile| profile.user_name),
                bound: true,
            })
        } else {
            Ok(PixivConnectionTestResponse {
                configured: true,
                status: "configured".to_owned(),
                pixiv_id: None,
                title: None,
                user_uid: profile.as_ref().map(|profile| profile.user_uid.clone()),
                user_name: profile.and_then(|profile| profile.user_name),
                bound: true,
            })
        }
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope { data: response }))
}

pub(crate) async fn post_test_deepseek(
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
