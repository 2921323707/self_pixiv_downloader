use axum::Json;
use axum::extract::{Path, State};

use crate::accounts::PixivAccountRepository;
use crate::api::AppState;
use crate::api::dto::{
    PixivAccountActivateRequest, PixivAccountDeleteResponse, PixivAccountResponse,
    PixivAccountsListResponse, pixiv_account_response,
};
use crate::api::error::{ApiEnvelope, ApiError};
use crate::api::runtime::prepare_db_path;
use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::settings::SettingsRepository;

pub(crate) async fn list_pixiv_accounts(
    State(state): State<AppState>,
) -> Result<Json<ApiEnvelope<PixivAccountsListResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let response = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let repo = PixivAccountRepository::new(&conn);
        let active = repo.get_active_public()?.map(pixiv_account_response);
        let items = repo
            .list_public()?
            .into_iter()
            .map(pixiv_account_response)
            .collect();
        Ok::<_, AppError>(PixivAccountsListResponse { items, active })
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope { data: response }))
}

pub(crate) async fn activate_pixiv_account(
    State(state): State<AppState>,
    Json(payload): Json<PixivAccountActivateRequest>,
) -> Result<Json<ApiEnvelope<PixivAccountResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let response = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let accounts = PixivAccountRepository::new(&conn);
        let account = accounts.set_active(&payload.user_uid)?;
        let cookie = serde_json::from_str::<String>(&account.cookie_json).map_err(|error| {
            AppError::new(
                ErrorCode::InternalError,
                format!("pixiv account cookie could not be decoded: {error}"),
            )
        })?;
        let settings = SettingsRepository::new(&conn);
        settings.upsert("pixiv_cookie", &serde_json::json!(cookie).to_string(), true)?;
        settings.upsert(
            "pixiv_active_account_uid",
            &serde_json::json!(account.user_uid).to_string(),
            false,
        )?;
        settings.upsert(
            "pixiv_active_account_name",
            &serde_json::json!(
                account
                    .user_name
                    .clone()
                    .unwrap_or_else(|| format!("Pixiv UID {}", account.user_uid))
            )
            .to_string(),
            false,
        )?;

        Ok::<_, AppError>(pixiv_account_response(
            accounts
                .get_active_public()?
                .ok_or_else(|| AppError::validation("pixiv account was not activated"))?,
        ))
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope { data: response }))
}

pub(crate) async fn delete_pixiv_account(
    State(state): State<AppState>,
    Path(user_uid): Path<String>,
) -> Result<Json<ApiEnvelope<PixivAccountDeleteResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let response = tokio::task::spawn_blocking(move || {
        prepare_db_path(&db_path)?;
        let conn = db::open(&db_path)?;
        let accounts = PixivAccountRepository::new(&conn);
        let was_active = accounts
            .get_public(&user_uid)?
            .is_some_and(|account| account.is_active);
        let deleted = accounts.delete(&user_uid)?;
        if deleted && was_active {
            let settings = SettingsRepository::new(&conn);
            settings.upsert("pixiv_cookie", &serde_json::json!("").to_string(), true)?;
            settings.upsert(
                "pixiv_active_account_uid",
                &serde_json::json!("").to_string(),
                false,
            )?;
            settings.upsert(
                "pixiv_active_account_name",
                &serde_json::json!("").to_string(),
                false,
            )?;
        }
        Ok::<_, AppError>(PixivAccountDeleteResponse { deleted })
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))??;

    Ok(Json(ApiEnvelope { data: response }))
}
