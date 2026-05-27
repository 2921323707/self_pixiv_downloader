use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

use crate::domain::R18Policy;
use crate::errors::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingValue {
    pub key: String,
    pub value_json: String,
    pub is_secret: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicSettingValue {
    pub key: String,
    pub value_json: String,
    pub is_secret: bool,
    pub updated_at: String,
}

pub struct SettingsRepository<'conn> {
    conn: &'conn Connection,
}

impl<'conn> SettingsRepository<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    pub fn upsert(&self, key: &str, value_json: &str, is_secret: bool) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO settings (key, value_json, is_secret, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                is_secret = excluded.is_secret,
                updated_at = excluded.updated_at",
            params![key, value_json, i64::from(is_secret)],
        )?;
        Ok(())
    }

    pub fn get_raw(&self, key: &str) -> Result<Option<SettingValue>, AppError> {
        self.conn
            .query_row(
                "SELECT key, value_json, is_secret, updated_at
                 FROM settings
                 WHERE key = ?1",
                params![key],
                |row| {
                    Ok(SettingValue {
                        key: row.get(0)?,
                        value_json: row.get(1)?,
                        is_secret: row.get::<_, i64>(2)? == 1,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn get_public(&self, key: &str) -> Result<Option<PublicSettingValue>, AppError> {
        Ok(self.get_raw(key)?.map(mask_setting))
    }

    pub fn list_public(&self) -> Result<Vec<PublicSettingValue>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT key, value_json, is_secret, updated_at
             FROM settings
             ORDER BY key",
        )?;
        let settings = stmt
            .query_map([], |row| {
                Ok(SettingValue {
                    key: row.get(0)?,
                    value_json: row.get(1)?,
                    is_secret: row.get::<_, i64>(2)? == 1,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut public = settings.into_iter().map(mask_setting).collect::<Vec<_>>();
        let timestamp = self.current_timestamp()?;
        for definition in SETTING_DEFINITIONS {
            if public.iter().any(|setting| setting.key == definition.key) {
                continue;
            }
            public.push(PublicSettingValue {
                key: definition.key.to_owned(),
                value_json: default_public_value_json(*definition).to_owned(),
                is_secret: definition.is_secret,
                updated_at: timestamp.clone(),
            });
        }
        public.sort_by(|left, right| left.key.cmp(&right.key));
        Ok(public)
    }

    pub fn upsert_known_json(
        &self,
        key: &str,
        value: &Value,
    ) -> Result<PublicSettingValue, AppError> {
        let definition = setting_definition(key)
            .ok_or_else(|| AppError::validation(format!("unknown setting key: {key}")))?;
        validate_setting_value(definition, value)?;

        if definition.is_secret && is_mask_sentinel(value) {
            return self
                .get_public(key)?
                .ok_or_else(|| AppError::validation("masked secret cannot create a new setting"));
        }

        self.upsert(key, &value.to_string(), definition.is_secret)?;
        self.get_public(key)?
            .ok_or_else(|| AppError::validation("setting was not saved"))
    }

    pub fn current_timestamp(&self) -> Result<String, AppError> {
        self.conn
            .query_row("SELECT datetime('now')", [], |row| row.get(0))
            .map_err(AppError::from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SettingDefinition {
    key: &'static str,
    is_secret: bool,
}

const SETTING_DEFINITIONS: &[SettingDefinition] = &[
    SettingDefinition {
        key: "download_base_path",
        is_secret: false,
    },
    SettingDefinition {
        key: "deepseek_base_url",
        is_secret: false,
    },
    SettingDefinition {
        key: "deepseek_model",
        is_secret: false,
    },
    SettingDefinition {
        key: "default_batch_count",
        is_secret: false,
    },
    SettingDefinition {
        key: "max_request_count",
        is_secret: false,
    },
    SettingDefinition {
        key: "r18_policy",
        is_secret: false,
    },
    SettingDefinition {
        key: "theme_id",
        is_secret: false,
    },
    SettingDefinition {
        key: "pixiv_cookie",
        is_secret: true,
    },
    SettingDefinition {
        key: "deepseek_api_key",
        is_secret: true,
    },
];

const THEME_IDS: &[&str] = &["cyan-studio", "sakura-light"];

fn setting_definition(key: &str) -> Option<SettingDefinition> {
    SETTING_DEFINITIONS
        .iter()
        .copied()
        .find(|definition| definition.key == key)
}

fn default_public_value_json(definition: SettingDefinition) -> &'static str {
    match definition.key {
        "download_base_path" => "\"~/Downloads/Pixiv Platform\"",
        "deepseek_base_url" => "\"https://api.deepseek.com\"",
        "deepseek_model" => "\"deepseek-v4-flash\"",
        "default_batch_count" => "20",
        "max_request_count" => "100",
        "r18_policy" => "\"exclude\"",
        "theme_id" => "\"cyan-studio\"",
        "pixiv_cookie" | "deepseek_api_key" => "\"***\"",
        _ => "null",
    }
}

fn mask_setting(setting: SettingValue) -> PublicSettingValue {
    PublicSettingValue {
        key: setting.key,
        value_json: if setting.is_secret {
            "\"***\"".to_owned()
        } else {
            setting.value_json
        },
        is_secret: setting.is_secret,
        updated_at: setting.updated_at,
    }
}

fn validate_setting_value(definition: SettingDefinition, value: &Value) -> Result<(), AppError> {
    match definition.key {
        "download_base_path" => {
            validate_download_base_path(value)?;
        }
        "deepseek_base_url" | "deepseek_model" => {
            non_empty_string(value, definition.key)?;
        }
        "theme_id" => {
            let theme = non_empty_string(value, definition.key)?;
            if !THEME_IDS.contains(&theme) {
                return Err(AppError::validation("theme_id is invalid"));
            }
        }
        "r18_policy" => {
            let policy = non_empty_string(value, definition.key)?;
            if R18Policy::from_api(policy).is_none() {
                return Err(AppError::validation("r18_policy is invalid"));
            }
        }
        "default_batch_count" | "max_request_count" => {
            let count = value.as_u64().ok_or_else(|| {
                AppError::validation(format!("{} must be a number", definition.key))
            })?;
            if count == 0 || count > 500 {
                return Err(AppError::validation(format!(
                    "{} must be between 1 and 500",
                    definition.key
                )));
            }
        }
        "pixiv_cookie" | "deepseek_api_key" => {
            if !is_mask_sentinel(value) {
                non_empty_string(value, definition.key)?;
            }
        }
        _ => return Err(AppError::validation("unknown setting key")),
    }
    Ok(())
}

fn non_empty_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, AppError> {
    let text = value
        .as_str()
        .ok_or_else(|| AppError::validation(format!("{key} must be a string")))?;
    if text.trim().is_empty() {
        return Err(AppError::validation(format!("{key} cannot be empty")));
    }
    Ok(text)
}

fn is_mask_sentinel(value: &Value) -> bool {
    value.as_str() == Some("***")
}

fn validate_download_base_path(value: &Value) -> Result<(), AppError> {
    let path = non_empty_string(value, "download_base_path")?.trim();
    if path.contains('\0') {
        return Err(AppError::validation(
            "download_base_path cannot contain NUL bytes",
        ));
    }
    if !(PathBuf::from(path).is_absolute() || path == "~" || path.starts_with("~/")) {
        return Err(AppError::validation(
            "download_base_path must be absolute or start with ~",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::db::open_in_memory;
    use crate::settings::SettingsRepository;

    #[test]
    fn req_sec_001_settings_repository_masks_secret_values() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        repo.upsert("pixiv_cookie", "\"placeholder-runtime-value\"", true)
            .unwrap();

        let raw = repo.get_raw("pixiv_cookie").unwrap().unwrap();
        let public = repo.get_public("pixiv_cookie").unwrap().unwrap();

        assert_eq!(raw.value_json, "\"placeholder-runtime-value\"");
        assert_eq!(public.value_json, "\"***\"");
        assert!(public.is_secret);
    }

    #[test]
    fn req_cfg_006_settings_repository_reads_default_theme_publicly() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        let theme = repo.get_public("theme_id").unwrap().unwrap();

        assert_eq!(theme.value_json, "\"cyan-studio\"");
        assert!(!theme.is_secret);
    }

    #[test]
    fn req_cfg_001_settings_repository_upserts_existing_value() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        repo.upsert("download_base_path", "\"/tmp/one\"", false)
            .unwrap();
        repo.upsert("download_base_path", "\"/tmp/two\"", false)
            .unwrap();

        let value = repo.get_raw("download_base_path").unwrap().unwrap();
        assert_eq!(value.value_json, "\"/tmp/two\"");
    }

    #[test]
    fn req_sec_001_settings_repository_masks_secrets_in_public_list() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);
        repo.upsert("deepseek_api_key", "\"placeholder-runtime-value\"", true)
            .unwrap();

        let public = repo.list_public().unwrap();
        let api_key = public
            .iter()
            .find(|setting| setting.key == "deepseek_api_key")
            .unwrap();

        assert_eq!(api_key.value_json, "\"***\"");
    }

    #[test]
    fn req_ui_004_settings_repository_lists_missing_known_secret_settings() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        let public = repo.list_public().unwrap();
        let pixiv_cookie = public
            .iter()
            .find(|setting| setting.key == "pixiv_cookie")
            .unwrap();
        let deepseek_key = public
            .iter()
            .find(|setting| setting.key == "deepseek_api_key")
            .unwrap();

        assert_eq!(pixiv_cookie.value_json, "\"***\"");
        assert!(pixiv_cookie.is_secret);
        assert_eq!(deepseek_key.value_json, "\"***\"");
        assert!(deepseek_key.is_secret);
    }

    #[test]
    fn req_cfg_003_settings_repository_reads_default_deepseek_model_publicly() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        let settings = repo.list_public().unwrap();
        let model = settings
            .iter()
            .find(|setting| setting.key == "deepseek_model")
            .unwrap();

        assert_eq!(model.value_json, "\"deepseek-v4-flash\"");
        assert!(!model.is_secret);
    }

    #[test]
    fn req_cfg_001_req_sec_001_settings_repository_saves_known_values_and_masks_secret() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        let theme = repo
            .upsert_known_json("theme_id", &serde_json::json!("cyan-studio"))
            .unwrap();
        let cookie = repo
            .upsert_known_json(
                "pixiv_cookie",
                &serde_json::json!("placeholder-runtime-value"),
            )
            .unwrap();
        let retained = repo
            .upsert_known_json("pixiv_cookie", &serde_json::json!("***"))
            .unwrap();

        assert_eq!(theme.value_json, "\"cyan-studio\"");
        assert_eq!(cookie.value_json, "\"***\"");
        assert_eq!(retained.value_json, "\"***\"");
        assert_eq!(
            repo.get_raw("pixiv_cookie").unwrap().unwrap().value_json,
            "\"placeholder-runtime-value\""
        );
    }

    #[test]
    fn req_cfg_001_settings_repository_rejects_unknown_or_invalid_values() {
        let conn = open_in_memory().unwrap();
        let repo = SettingsRepository::new(&conn);

        assert!(
            repo.upsert_known_json("unknown", &serde_json::json!("value"))
                .is_err()
        );
        assert!(
            repo.upsert_known_json("r18_policy", &serde_json::json!("bad-policy"))
                .is_err()
        );
        assert!(
            repo.upsert_known_json("theme_id", &serde_json::json!("bad-theme"))
                .is_err()
        );
        assert!(
            repo.upsert_known_json("theme_id", &serde_json::json!("sakura-light"))
                .is_ok()
        );
        assert!(
            repo.upsert_known_json("max_request_count", &serde_json::json!(0))
                .is_err()
        );
        assert!(
            repo.upsert_known_json("download_base_path", &serde_json::json!("relative/path"))
                .is_err()
        );
        assert!(
            repo.upsert_known_json(
                "download_base_path",
                &serde_json::json!("~/Downloads/Pixiv Platform")
            )
            .is_ok()
        );
    }
}
