use std::env;
use std::fs;
use std::path::{Path as FsPath, PathBuf};

use crate::ai::DeepSeekConfig;
use crate::domain::R18Policy;
use crate::errors::{AppError, ErrorCode};
use crate::settings::SettingsRepository;

pub(crate) const DEFAULT_DOWNLOAD_BASE_PATH: &str = "~/Downloads/Pixiv Platform";
pub(crate) const LEGACY_PROJECT_DOWNLOAD_BASE_PATH: &str = "project:output";
pub(crate) const LEGACY_DEFAULT_DOWNLOAD_BASE_PATH: &str = "~/pixiv_downloads/";
pub(crate) const DEFAULT_BATCH_COUNT: u32 = 20;
pub(crate) const DEFAULT_MAX_REQUEST_COUNT: u32 = 100;
pub(crate) const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
pub(crate) const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-v4-flash";

pub(crate) fn prepare_local_paths(
    db_path: &PathBuf,
    download_root: &PathBuf,
) -> Result<(), AppError> {
    fs::create_dir_all(download_root)?;
    prepare_db_path(db_path)
}

pub(crate) fn prepare_db_path(db_path: &PathBuf) -> Result<(), AppError> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSettings {
    pub(crate) download_root: PathBuf,
    pub(crate) pixiv_cookie: Option<String>,
}

pub(crate) fn resolve_runtime_settings(
    conn: &rusqlite::Connection,
    fallback_download_root: &FsPath,
) -> Result<RuntimeSettings, AppError> {
    let settings = SettingsRepository::new(conn);
    Ok(RuntimeSettings {
        download_root: resolve_download_root(&settings, fallback_download_root)?,
        pixiv_cookie: resolve_pixiv_cookie(&settings)?,
    })
}

pub(crate) fn resolve_pixiv_cookie(
    settings: &SettingsRepository<'_>,
) -> Result<Option<String>, AppError> {
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

pub(crate) fn resolve_deepseek_config(
    settings: &SettingsRepository<'_>,
) -> Result<DeepSeekConfig, AppError> {
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

pub(crate) fn resolve_secret_setting(
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

pub(crate) fn resolve_string_setting(
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

pub(crate) fn resolve_download_root(
    settings: &SettingsRepository<'_>,
    fallback_download_root: &FsPath,
) -> Result<PathBuf, AppError> {
    let Some(setting) = settings.get_raw("download_base_path")? else {
        return Ok(fallback_download_root.to_path_buf());
    };
    let raw = setting_string(&setting.value_json, "download_base_path")?;
    let trimmed = raw.trim();
    if trimmed == DEFAULT_DOWNLOAD_BASE_PATH
        || trimmed == LEGACY_PROJECT_DOWNLOAD_BASE_PATH
        || trimmed == LEGACY_DEFAULT_DOWNLOAD_BASE_PATH
    {
        return Ok(fallback_download_root.to_path_buf());
    }
    expand_download_root(trimmed)
}

pub(crate) fn expand_download_root(value: &str) -> Result<PathBuf, AppError> {
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
            "download_base_path must be absolute or start with ~",
        ));
    }
    Ok(path)
}

fn home_dir() -> Result<String, AppError> {
    env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| {
            AppError::validation(
                "HOME or USERPROFILE is required to expand download_base_path starting with ~",
            )
        })
}

pub(crate) fn setting_string(raw_json: &str, key: &str) -> Result<String, AppError> {
    serde_json::from_str::<String>(raw_json)
        .map_err(|_| AppError::validation(format!("{key} must be stored as a JSON string")))
}

pub(crate) fn setting_u32_or(
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

pub(crate) fn setting_r18_policy_or(
    settings: &SettingsRepository<'_>,
    fallback: R18Policy,
) -> Result<R18Policy, AppError> {
    let Some(setting) = settings.get_raw("r18_policy")? else {
        return Ok(fallback);
    };
    let value = setting_string(&setting.value_json, "r18_policy")?;
    R18Policy::from_api(&value).ok_or_else(|| AppError::validation("r18_policy is invalid"))
}

pub(crate) fn default_download_root() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|home| home.join("Downloads/Pixiv Platform"))
        .unwrap_or_else(|| PathBuf::from("Pixiv Platform"))
}
