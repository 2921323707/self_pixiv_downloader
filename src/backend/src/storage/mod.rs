use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::{AppError, ErrorCode};

#[derive(Debug, Clone)]
pub struct StoragePlanner {
    root: PathBuf,
}

impl StoragePlanner {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn original_path(
        &self,
        pixiv_id: &str,
        page_index: u32,
        extension: Option<&str>,
    ) -> Result<PathBuf, AppError> {
        let safe_id = sanitize_segment(pixiv_id)?;
        let safe_ext = sanitize_extension(extension.unwrap_or("bin"))?;
        Ok(self
            .root
            .join("originals")
            .join(&safe_id)
            .join(format!("{safe_id}_p{page_index}.{safe_ext}")))
    }

    pub fn write_atomic(&self, final_path: &Path, bytes: &[u8]) -> Result<(), AppError> {
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = temp_path_for(final_path);
        fs::write(&tmp_path, bytes).map_err(AppError::from)?;
        fs::rename(&tmp_path, final_path).map_err(|err| {
            let _ = fs::remove_file(&tmp_path);
            AppError::new(ErrorCode::FilesystemWriteFailed, err.to_string())
        })?;
        Ok(())
    }
}

fn sanitize_segment(value: &str) -> Result<String, AppError> {
    if value.is_empty() {
        return Err(AppError::validation("path segment cannot be empty"));
    }

    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::validation(
            "path segment contains unsafe characters",
        ));
    }

    Ok(value.to_owned())
}

fn sanitize_extension(value: &str) -> Result<String, AppError> {
    let lower = value.trim_start_matches('.').to_ascii_lowercase();
    if lower.is_empty() {
        return Ok("bin".to_owned());
    }

    if !lower.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AppError::validation("extension contains unsafe characters"));
    }

    Ok(lower)
}

fn temp_path_for(final_path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let file_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download.bin");
    final_path.with_file_name(format!(".{file_name}.{nanos}.tmp"))
}

#[cfg(test)]
mod tests {
    use super::StoragePlanner;

    #[test]
    fn req_dl_006_plans_grouped_original_path() {
        let planner = StoragePlanner::new("/tmp/pixiv_downloads");
        let path = planner.original_path("123456", 2, Some("jpg")).unwrap();
        assert_eq!(
            path.to_string_lossy(),
            "/tmp/pixiv_downloads/originals/123456/123456_p2.jpg"
        );
    }

    #[test]
    fn req_sec_002_rejects_unsafe_pixiv_id_path_segments() {
        let planner = StoragePlanner::new("/tmp/pixiv_downloads");
        let error = planner.original_path("../123", 0, Some("jpg")).unwrap_err();
        assert_eq!(error.code.as_str(), "VALIDATION_ERROR");
    }
}
