use std::sync::Arc;
use std::time::Instant;

use axum::Json;
use axum::extract::State;

use crate::accounts::PixivAccountRepository;
use crate::api::AppState;
use crate::api::dto::{
    RuntimePixivAccountReadinessResponse, RuntimeReadinessActionResponse,
    RuntimeReadinessCheckResponse, RuntimeReadinessResponse, pixiv_account_response,
};
use crate::api::error::{ApiEnvelope, ApiError};
use crate::api::runtime::{prepare_local_paths, resolve_deepseek_config, resolve_runtime_settings};
use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::settings::SettingsRepository;

pub(crate) async fn get_runtime_readiness(
    State(state): State<AppState>,
) -> Result<Json<ApiEnvelope<RuntimeReadinessResponse>>, ApiError> {
    let db_path = state.inner.db_path.clone();
    let download_root = state.inner.download_root.clone();
    let pixiv_factory = Arc::clone(&state.inner.pixiv_client_factory);
    let ai_factory = Arc::clone(&state.inner.ai_client_factory);

    let response = tokio::task::spawn_blocking(move || {
        let backend = match prepare_local_paths(&db_path, &download_root)
            .and_then(|_| db::open(&db_path).map(|_| ()))
        {
            Ok(()) => ok_check("ok", "Backend is ready."),
            Err(error) => error_check(
                "failed",
                "Backend storage is not ready.",
                Some("Check the local database path and app logs, then restart Pixiv Platform."),
                Some(action("Open Settings", Some("/settings"), None)),
                error,
            ),
        };

        let pixiv_network_started_at = Instant::now();
        let pixiv_network = match pixiv_factory.probe_network() {
            Ok(()) => {
                let mut check = ok_check("ok", "Pixiv network is reachable.");
                check.latency_ms = Some(pixiv_network_started_at.elapsed().as_millis());
                check
            }
            Err(error) => error_check(
                "unreachable",
                "Pixiv network is unreachable.",
                Some("Network unreachable. Please enable TUN mode and retry."),
                Some(action("Retry", None, Some("retry"))),
                error,
            ),
        };

        let (pixiv_account, deepseek) = match db::open(&db_path) {
            Ok(conn) => {
                let settings = SettingsRepository::new(&conn);
                let runtime = resolve_runtime_settings(&conn, &download_root);
                (
                    match runtime {
                        Ok(runtime) => resolve_pixiv_account_readiness(
                            &conn,
                            &pixiv_factory,
                            runtime.pixiv_cookie,
                        ),
                        Err(error) => account_error_check(
                            "failed",
                            "Pixiv account settings could not be read.",
                            Some("Open Settings and save the Pixiv cookie again."),
                            Some(action("Open Settings", Some("/settings"), None)),
                            error,
                        ),
                    },
                    resolve_deepseek_readiness(&settings, &ai_factory),
                )
            }
            Err(error) => {
                let app_error = AppError::from(error);
                (
                    account_error_check(
                        "unknown",
                        "Pixiv account status could not be checked.",
                        Some("Backend storage is unavailable. Restart the app and check logs."),
                        None,
                        app_error.clone(),
                    ),
                    error_check(
                        "unknown",
                        "DeepSeek status could not be checked.",
                        Some("Backend storage is unavailable. Restart the app and check logs."),
                        None,
                        app_error,
                    ),
                )
            }
        };

        RuntimeReadinessResponse {
            backend,
            pixiv_network,
            pixiv_account,
            deepseek,
        }
    })
    .await
    .map_err(|error| AppError::new(ErrorCode::InternalError, error.to_string()))?;

    Ok(Json(ApiEnvelope { data: response }))
}

fn resolve_pixiv_account_readiness(
    conn: &rusqlite::Connection,
    pixiv_factory: &Arc<dyn crate::api::PixivClientFactory>,
    pixiv_cookie: Option<String>,
) -> RuntimePixivAccountReadinessResponse {
    let Some(cookie) = pixiv_cookie else {
        return RuntimePixivAccountReadinessResponse {
            ok: false,
            status: "missing".to_owned(),
            message: "Pixiv cookie is not connected.".to_owned(),
            recommendation: Some(
                "Bind Pixiv from Home or Settings to enable downloads.".to_owned(),
            ),
            action: Some(action("Bind Pixiv", None, Some("bind_pixiv"))),
            error_code: Some(ErrorCode::MissingPixivCookie.as_str().to_owned()),
            latency_ms: None,
            account: PixivAccountRepository::new(conn)
                .get_active_public()
                .ok()
                .flatten()
                .map(pixiv_account_response),
        };
    };

    let client = match pixiv_factory.create_with_cookie(Some(&cookie)) {
        Ok(client) => client,
        Err(error) => {
            return account_error_check(
                "failed",
                "Pixiv cookie could not be used.",
                Some("Bind Pixiv again or switch to another saved account."),
                Some(action("Bind Pixiv", None, Some("bind_pixiv"))),
                error,
            );
        }
    };

    match client.fetch_current_user_profile() {
        Ok(profile) => {
            let account = PixivAccountRepository::new(conn)
                .upsert_active(&profile.user_uid, profile.user_name.as_deref(), &cookie)
                .ok();
            if let Some(account) = account.as_ref() {
                let settings = SettingsRepository::new(conn);
                let _ = settings.upsert(
                    "pixiv_active_account_uid",
                    &serde_json::json!(account.user_uid).to_string(),
                    false,
                );
                let _ = settings.upsert(
                    "pixiv_active_account_name",
                    &serde_json::json!(account_label(
                        account.user_name.as_deref(),
                        &account.user_uid
                    ))
                    .to_string(),
                    false,
                );
            }
            let account = account.map(pixiv_account_response);
            RuntimePixivAccountReadinessResponse {
                ok: true,
                status: "bound".to_owned(),
                message: account
                    .as_ref()
                    .map(|account| account_label(account.user_name.as_deref(), &account.user_uid))
                    .map(|label| format!("Pixiv account is bound as {label}."))
                    .unwrap_or_else(|| "Pixiv account is bound.".to_owned()),
                recommendation: None,
                action: Some(action("Switch Account", Some("/settings"), None)),
                error_code: None,
                latency_ms: None,
                account,
            }
        }
        Err(error) => account_error_check(
            "invalid",
            "Pixiv cookie is not connected.",
            Some("Bind Pixiv again; if Pixiv cannot load, enable TUN mode first."),
            Some(action("Bind Pixiv", None, Some("bind_pixiv"))),
            error,
        ),
    }
}

fn resolve_deepseek_readiness(
    settings: &SettingsRepository<'_>,
    ai_factory: &Arc<dyn crate::api::AiClientFactory>,
) -> RuntimeReadinessCheckResponse {
    let config = match resolve_deepseek_config(settings) {
        Ok(config) => config,
        Err(error) => {
            return error_check(
                "missing",
                "DeepSeek is not configured.",
                Some("Add a DeepSeek API key in Settings to enable smart retrieval."),
                Some(action("Open Settings", Some("/settings"), None)),
                error,
            );
        }
    };

    let model = config.model.clone();
    match ai_factory
        .create(config)
        .and_then(|client| client.test_connection())
    {
        Ok(status) => RuntimeReadinessCheckResponse {
            ok: status.status == "ok",
            status: status.status.clone(),
            message: if status.status == "ok" {
                format!("DeepSeek model {} is reachable.", status.model)
            } else {
                format!(
                    "DeepSeek responded, but model {} was not listed.",
                    status.model
                )
            },
            recommendation: None,
            action: Some(action("Open Settings", Some("/settings"), None)),
            error_code: None,
            latency_ms: None,
        },
        Err(error) => error_check(
            "failed",
            format!("DeepSeek model {model} could not be reached."),
            Some("Check the DeepSeek key, base URL, model, and network in Settings."),
            Some(action("Open Settings", Some("/settings"), None)),
            error,
        ),
    }
}

fn ok_check(status: &str, message: &str) -> RuntimeReadinessCheckResponse {
    RuntimeReadinessCheckResponse {
        ok: true,
        status: status.to_owned(),
        message: message.to_owned(),
        recommendation: None,
        action: None,
        error_code: None,
        latency_ms: None,
    }
}

fn error_check(
    status: &str,
    message: impl Into<String>,
    recommendation: Option<&str>,
    action: Option<RuntimeReadinessActionResponse>,
    error: AppError,
) -> RuntimeReadinessCheckResponse {
    RuntimeReadinessCheckResponse {
        ok: false,
        status: status.to_owned(),
        message: message.into(),
        recommendation: recommendation.map(str::to_owned),
        action,
        error_code: Some(error.code.as_str().to_owned()),
        latency_ms: None,
    }
}

fn account_error_check(
    status: &str,
    message: &str,
    recommendation: Option<&str>,
    action: Option<RuntimeReadinessActionResponse>,
    error: AppError,
) -> RuntimePixivAccountReadinessResponse {
    RuntimePixivAccountReadinessResponse {
        ok: false,
        status: status.to_owned(),
        message: message.to_owned(),
        recommendation: recommendation.map(str::to_owned),
        action,
        error_code: Some(error.code.as_str().to_owned()),
        latency_ms: None,
        account: None,
    }
}

fn action(label: &str, href: Option<&str>, action: Option<&str>) -> RuntimeReadinessActionResponse {
    RuntimeReadinessActionResponse {
        label: label.to_owned(),
        href: href.map(str::to_owned),
        action: action.map(str::to_owned),
    }
}

fn account_label(user_name: Option<&str>, user_uid: &str) -> String {
    user_name
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("Pixiv UID {user_uid}"))
}
