use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, ToSql, params, params_from_iter};

use crate::domain::{ImageCategory, ImageSource};
use crate::errors::{AppError, ErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub struct NewImageRecord {
    pub image_id: String,
    pub pixiv_id: String,
    pub page_index: u32,
    pub author_uid: Option<String>,
    pub title: Option<String>,
    pub category: ImageCategory,
    pub local_path: String,
    pub thumbnail_path: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub map_x: Option<f64>,
    pub map_y: Option<f64>,
    pub downloaded_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageRecord {
    pub image_id: String,
    pub pixiv_id: String,
    pub page_index: u32,
    pub author_uid: Option<String>,
    pub title: Option<String>,
    pub category: ImageCategory,
    pub local_path: String,
    pub thumbnail_path: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub map_x: Option<f64>,
    pub map_y: Option<f64>,
    pub downloaded_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSourceRecord {
    pub image_id: String,
    pub source: ImageSource,
    pub task_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageListQuery {
    pub tag: Option<String>,
    pub category: Option<ImageCategory>,
    pub author_uid: Option<String>,
    pub source: Option<ImageSource>,
    pub r18_visibility: ImageR18Visibility,
    pub limit: usize,
    pub cursor_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageR18Visibility {
    Include,
    Exclude,
    OnlyR18,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageListItem {
    pub image: ImageRecord,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageListPage {
    pub items: Vec<ImageListItem>,
    pub next_cursor_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageFile {
    pub path: PathBuf,
    pub content_type: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageDeleteOutcome {
    pub image_id: String,
    pub pixiv_id: String,
    pub page_index: u32,
    pub file_deleted: bool,
    pub file_missing: bool,
}

pub struct ImageRepository<'conn> {
    conn: &'conn Connection,
}

impl<'conn> ImageRepository<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, image: &NewImageRecord) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO images (
                image_id, pixiv_id, page_index, author_uid, title, category,
                local_path, thumbnail_path, width, height, map_x, map_y,
                downloaded_at, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11, ?12,
                ?13, datetime('now'), datetime('now')
            )",
            params![
                image.image_id,
                image.pixiv_id,
                image.page_index,
                image.author_uid,
                image.title,
                image.category.as_str(),
                image.local_path,
                image.thumbnail_path,
                image.width,
                image.height,
                image.map_x,
                image.map_y,
                image.downloaded_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_from_download(&self, image: &NewImageRecord) -> Result<(), AppError> {
        let affected = self.conn.execute(
            "UPDATE images
             SET author_uid = ?2,
                 title = ?3,
                 category = ?4,
                 local_path = ?5,
                 thumbnail_path = ?6,
                 width = ?7,
                 height = ?8,
                 map_x = ?9,
                 map_y = ?10,
                 downloaded_at = ?11,
                 updated_at = datetime('now')
             WHERE image_id = ?1",
            params![
                image.image_id,
                image.author_uid,
                image.title,
                image.category.as_str(),
                image.local_path,
                image.thumbnail_path,
                image.width,
                image.height,
                image.map_x,
                image.map_y,
                image.downloaded_at,
            ],
        )?;

        if affected == 0 {
            return Err(AppError::new(
                crate::errors::ErrorCode::InternalError,
                format!("image {} does not exist", image.image_id),
            ));
        }

        Ok(())
    }

    pub fn find_by_id(&self, image_id: &str) -> Result<Option<ImageRecord>, AppError> {
        self.conn
            .query_row(
                "SELECT
                    image_id, pixiv_id, page_index, author_uid, title, category,
                    local_path, thumbnail_path, width, height, map_x, map_y,
                    downloaded_at, created_at, updated_at
                 FROM images
                 WHERE image_id = ?1",
                params![image_id],
                row_to_image,
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn find_by_pixiv_page(
        &self,
        pixiv_id: &str,
        page_index: u32,
    ) -> Result<Option<ImageRecord>, AppError> {
        self.conn
            .query_row(
                "SELECT
                    image_id, pixiv_id, page_index, author_uid, title, category,
                    local_path, thumbnail_path, width, height, map_x, map_y,
                    downloaded_at, created_at, updated_at
                 FROM images
                 WHERE pixiv_id = ?1 AND page_index = ?2",
                params![pixiv_id, page_index],
                row_to_image,
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn replace_tags(&self, image_id: &str, tags: &[String]) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM image_tags WHERE image_id = ?1",
            params![image_id],
        )?;
        for tag in tags {
            tx.execute(
                "INSERT INTO image_tags (image_id, tag, created_at)
                 VALUES (?1, ?2, datetime('now'))",
                params![image_id, tag],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn tags_for_image(&self, image_id: &str) -> Result<Vec<String>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM image_tags WHERE image_id = ?1 ORDER BY tag")?;
        let tags = stmt
            .query_map(params![image_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn add_source(
        &self,
        image_id: &str,
        source: ImageSource,
        task_id: Option<&str>,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO image_sources (image_id, source, task_id, created_at)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![image_id, source.as_str(), task_id],
        )?;
        Ok(())
    }

    pub fn sources_for_image(&self, image_id: &str) -> Result<Vec<ImageSourceRecord>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT image_id, source, task_id, created_at
             FROM image_sources
             WHERE image_id = ?1
             ORDER BY created_at, source",
        )?;
        let sources = stmt
            .query_map(params![image_id], |row| {
                let source_value: String = row.get(1)?;
                let source = ImageSource::from_db(&source_value).ok_or_else(|| {
                    rusqlite::Error::FromSqlConversionFailure(
                        1,
                        rusqlite::types::Type::Text,
                        format!("invalid image source: {source_value}").into(),
                    )
                })?;
                Ok(ImageSourceRecord {
                    image_id: row.get(0)?,
                    source,
                    task_id: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sources)
    }

    pub fn list_images(&self, query: &ImageListQuery) -> Result<ImageListPage, AppError> {
        let limit = query.limit.clamp(1, 100);
        let fetch_limit = limit + 1;
        let mut sql = String::from(
            "SELECT
                i.image_id, i.pixiv_id, i.page_index, i.author_uid, i.title, i.category,
                i.local_path, i.thumbnail_path, i.width, i.height, i.map_x, i.map_y,
                i.downloaded_at, i.created_at, i.updated_at
             FROM images i
             WHERE 1 = 1",
        );
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(tag) = query.tag.as_ref().filter(|value| !value.trim().is_empty()) {
            sql.push_str(
                " AND EXISTS (
                    SELECT 1 FROM image_tags t
                    WHERE t.image_id = i.image_id AND t.tag = ?
                 )",
            );
            values.push(Box::new(tag.trim().to_owned()));
        }

        if let Some(category) = query.category {
            sql.push_str(" AND i.category = ?");
            values.push(Box::new(category.as_str().to_owned()));
        }

        if let Some(author_uid) = query
            .author_uid
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            sql.push_str(" AND i.author_uid = ?");
            values.push(Box::new(author_uid.trim().to_owned()));
        }

        if let Some(source) = query.source {
            sql.push_str(
                " AND EXISTS (
                    SELECT 1 FROM image_sources s
                    WHERE s.image_id = i.image_id AND s.source = ?
                 )",
            );
            values.push(Box::new(source.as_str().to_owned()));
        }

        match query.r18_visibility {
            ImageR18Visibility::Include => {}
            ImageR18Visibility::Exclude => {
                sql.push_str(" AND i.category = 'normal'");
            }
            ImageR18Visibility::OnlyR18 => {
                sql.push_str(" AND i.category IN ('r18', 'nsfw')");
            }
        }

        sql.push_str(" ORDER BY i.created_at DESC, i.image_id DESC LIMIT ? OFFSET ?");
        values.push(Box::new(fetch_limit as i64));
        values.push(Box::new(query.cursor_offset as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let images = stmt
            .query_map(
                params_from_iter(values.iter().map(|value| value.as_ref() as &dyn ToSql)),
                row_to_image,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        let has_next = images.len() > limit;
        let items = images
            .into_iter()
            .take(limit)
            .map(|image| {
                let tags = self.tags_for_image(&image.image_id)?;
                Ok(ImageListItem { image, tags })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(ImageListPage {
            items,
            next_cursor_offset: has_next.then_some(query.cursor_offset + limit),
        })
    }

    pub fn current_timestamp(&self) -> Result<String, AppError> {
        self.conn
            .query_row("SELECT datetime('now')", [], |row| row.get(0))
            .map_err(AppError::from)
    }

    pub fn delete_index(&self, image_id: &str) -> Result<bool, AppError> {
        let affected = self
            .conn
            .execute("DELETE FROM images WHERE image_id = ?1", params![image_id])?;
        Ok(affected > 0)
    }
}

pub fn preview_url_for(image_id: &str) -> String {
    format!("/api/images/{}/file", encode_path_segment(image_id))
}

pub fn resolve_image_file(
    image: &ImageRecord,
    allowed_roots: &[PathBuf],
) -> Result<ImageFile, AppError> {
    let raw_path = PathBuf::from(&image.local_path);
    if !raw_path.is_absolute() {
        return Err(AppError::validation("stored image path must be absolute"));
    }

    let canonical_path = raw_path
        .canonicalize()
        .map_err(|_| AppError::new(ErrorCode::PixivNotFound, "image file not found"))?;
    let metadata = fs::metadata(&canonical_path)
        .map_err(|_| AppError::new(ErrorCode::PixivNotFound, "image file not found"))?;
    if !metadata.is_file() {
        return Err(AppError::new(
            ErrorCode::PixivNotFound,
            "image file not found",
        ));
    }

    let canonical_roots = allowed_roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .collect::<Vec<_>>();
    if canonical_roots.is_empty()
        || !canonical_roots
            .iter()
            .any(|root| canonical_path.starts_with(root))
    {
        return Err(AppError::validation(
            "stored image path is outside allowed download roots",
        ));
    }

    Ok(ImageFile {
        content_type: content_type_for_path(&canonical_path),
        path: canonical_path,
    })
}

pub fn delete_image_file_and_index(
    repo: &ImageRepository<'_>,
    image_id: &str,
    allowed_roots: &[PathBuf],
) -> Result<ImageDeleteOutcome, AppError> {
    let image = repo
        .find_by_id(image_id)?
        .ok_or_else(|| AppError::new(ErrorCode::PixivNotFound, "image not found"))?;
    let delete_path = resolve_deletable_image_path(&image.local_path, allowed_roots)?;
    let mut file_deleted = false;
    let mut file_missing = false;

    if let Some(path) = delete_path {
        match fs::remove_file(&path) {
            Ok(()) => {
                file_deleted = true;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                file_missing = true;
            }
            Err(error) => {
                return Err(AppError::new(
                    ErrorCode::FilesystemWriteFailed,
                    format!("image file could not be deleted: {error}"),
                ));
            }
        }
    } else {
        file_missing = true;
    }

    repo.delete_index(&image.image_id)?;

    Ok(ImageDeleteOutcome {
        image_id: image.image_id,
        pixiv_id: image.pixiv_id,
        page_index: image.page_index,
        file_deleted,
        file_missing,
    })
}

fn resolve_deletable_image_path(
    local_path: &str,
    allowed_roots: &[PathBuf],
) -> Result<Option<PathBuf>, AppError> {
    let raw_path = PathBuf::from(local_path);
    if !raw_path.is_absolute() {
        return Err(AppError::validation("stored image path must be absolute"));
    }

    let canonical_roots = canonical_allowed_roots(allowed_roots);
    if canonical_roots.is_empty() {
        return Err(AppError::validation(
            "stored image path is outside allowed download roots",
        ));
    }

    if raw_path.exists() {
        let canonical_path = raw_path.canonicalize().map_err(|_| {
            AppError::new(ErrorCode::PixivNotFound, "image file could not be resolved")
        })?;
        let metadata = fs::metadata(&canonical_path).map_err(|_| {
            AppError::new(ErrorCode::PixivNotFound, "image file could not be resolved")
        })?;
        if !metadata.is_file() {
            return Err(AppError::validation("stored image path is not a file"));
        }
        ensure_path_inside_roots(&canonical_path, &canonical_roots)?;
        return Ok(Some(canonical_path));
    }

    let Some(parent) = raw_path.parent() else {
        return Err(AppError::validation("stored image path is invalid"));
    };
    let canonical_parent = parent.canonicalize().map_err(|_| {
        AppError::new(
            ErrorCode::PixivNotFound,
            "image file and parent directory were not found",
        )
    })?;
    ensure_path_inside_roots(&canonical_parent, &canonical_roots)?;
    Ok(None)
}

fn canonical_allowed_roots(allowed_roots: &[PathBuf]) -> Vec<PathBuf> {
    allowed_roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .collect()
}

fn ensure_path_inside_roots(path: &Path, canonical_roots: &[PathBuf]) -> Result<(), AppError> {
    if canonical_roots.iter().any(|root| path.starts_with(root)) {
        Ok(())
    } else {
        Err(AppError::validation(
            "stored image path is outside allowed download roots",
        ))
    }
}

fn row_to_image(row: &rusqlite::Row<'_>) -> Result<ImageRecord, rusqlite::Error> {
    let category_value: String = row.get(5)?;
    let category = ImageCategory::from_db(&category_value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            format!("invalid image category: {category_value}").into(),
        )
    })?;

    Ok(ImageRecord {
        image_id: row.get(0)?,
        pixiv_id: row.get(1)?,
        page_index: row.get(2)?,
        author_uid: row.get(3)?,
        title: row.get(4)?,
        category,
        local_path: row.get(6)?,
        thumbnail_path: row.get(7)?,
        width: row.get(8)?,
        height: row.get(9)?,
        map_x: row.get(10)?,
        map_y: row.get(11)?,
        downloaded_at: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("avif") => "image/avif",
        _ => "application/octet-stream",
    }
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rusqlite::{Error as SqliteError, ErrorCode as SqliteErrorCode};

    use crate::db::open_in_memory;
    use crate::domain::{ImageCategory, ImageSource};
    use crate::images::{
        ImageListQuery, ImageR18Visibility, ImageRepository, NewImageRecord,
        delete_image_file_and_index, preview_url_for, resolve_image_file,
    };

    #[test]
    fn req_img_001_repository_inserts_and_queries_image_metadata() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);

        repo.insert(&sample_image("image-1", "144920810", 0))
            .unwrap();
        let image = repo.find_by_pixiv_page("144920810", 0).unwrap().unwrap();

        assert_eq!(image.image_id, "image-1");
        assert_eq!(image.title.as_deref(), Some("おでかけ"));
        assert_eq!(image.category, ImageCategory::Normal);
        assert_eq!(image.width, Some(1062));
        assert_eq!(image.height, Some(1500));
    }

    #[test]
    fn req_dl_006_repository_finds_existing_pixiv_page_for_dedupe() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);

        repo.insert(&sample_image("image-1", "144920810", 0))
            .unwrap();

        assert!(repo.find_by_pixiv_page("144920810", 0).unwrap().is_some());
        assert!(repo.find_by_pixiv_page("144920810", 1).unwrap().is_none());
    }

    #[test]
    fn req_dl_006_repository_rejects_duplicate_pixiv_page_identity() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);

        repo.insert(&sample_image("image-1", "144920810", 0))
            .unwrap();
        let duplicate = repo.insert(&sample_image("image-2", "144920810", 0));

        assert!(matches!(
            duplicate.map_err(|err| err.code),
            Err(crate::errors::ErrorCode::SqliteError)
        ));
    }

    #[test]
    fn req_img_001_repository_replaces_and_reads_tags() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        repo.insert(&sample_image("image-1", "144920810", 0))
            .unwrap();

        repo.replace_tags("image-1", &["blue hair".to_owned(), "cyberpunk".to_owned()])
            .unwrap();
        repo.replace_tags("image-1", &["おでかけ".to_owned()])
            .unwrap();

        assert_eq!(repo.tags_for_image("image-1").unwrap(), vec!["おでかけ"]);
    }

    #[test]
    fn req_img_001_repository_rejects_tags_for_missing_image() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);

        let result = repo.replace_tags("missing", &["tag".to_owned()]);

        assert!(matches!(
            result,
            Err(crate::errors::AppError {
                code: crate::errors::ErrorCode::SqliteError,
                ..
            })
        ));
    }

    #[test]
    fn req_img_001_repository_records_source_history_once_per_identity() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        repo.insert(&sample_image("image-1", "144920810", 0))
            .unwrap();

        repo.add_source("image-1", ImageSource::Single, None)
            .unwrap();
        repo.add_source("image-1", ImageSource::Single, None)
            .unwrap();
        repo.add_source("image-1", ImageSource::Smart, None)
            .unwrap();

        let sources = repo.sources_for_image("image-1").unwrap();
        assert_eq!(sources.len(), 2);
        assert!(
            sources
                .iter()
                .any(|source| source.source == ImageSource::Single)
        );
        assert!(
            sources
                .iter()
                .any(|source| source.source == ImageSource::Smart)
        );
    }

    #[test]
    fn req_img_002_req_img_003_repository_lists_images_with_filters_and_cursor() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        repo.insert(&sample_image("image-1", "144920810", 0))
            .unwrap();
        repo.replace_tags("image-1", &["cyan".to_owned(), "girl".to_owned()])
            .unwrap();
        repo.add_source("image-1", ImageSource::Single, None)
            .unwrap();
        let mut r18 = sample_image("image-2", "144920811", 0);
        r18.category = ImageCategory::R18;
        r18.author_uid = Some("author-r18".to_owned());
        repo.insert(&r18).unwrap();
        repo.replace_tags("image-2", &["cyan".to_owned()]).unwrap();
        repo.add_source("image-2", ImageSource::Smart, None)
            .unwrap();

        let page = repo
            .list_images(&ImageListQuery {
                tag: Some("cyan".to_owned()),
                category: None,
                author_uid: None,
                source: Some(ImageSource::Single),
                r18_visibility: ImageR18Visibility::Exclude,
                limit: 1,
                cursor_offset: 0,
            })
            .unwrap();

        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].image.image_id, "image-1");
        assert_eq!(page.items[0].tags, vec!["cyan", "girl"]);

        let first_page = repo
            .list_images(&ImageListQuery {
                tag: Some("cyan".to_owned()),
                category: None,
                author_uid: None,
                source: None,
                r18_visibility: ImageR18Visibility::Include,
                limit: 1,
                cursor_offset: 0,
            })
            .unwrap();
        assert_eq!(first_page.items.len(), 1);
        assert_eq!(first_page.next_cursor_offset, Some(1));
    }

    #[test]
    fn req_img_001_repository_rejects_source_for_missing_image() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);

        let result = repo.add_source("missing", ImageSource::Single, None);

        assert!(matches!(
            result,
            Err(crate::errors::AppError {
                code: crate::errors::ErrorCode::SqliteError,
                ..
            })
        ));
    }

    #[test]
    fn req_img_001_repository_checks_constraints_are_sqlite_backed() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        let mut invalid = sample_image("image-1", "144920810", 0);
        invalid.local_path = String::new();

        repo.insert(&invalid).unwrap();
        let duplicate = repo.insert(&sample_image("image-2", "144920810", 0));

        assert!(matches!(
            duplicate,
            Err(crate::errors::AppError {
                code: crate::errors::ErrorCode::SqliteError,
                ..
            })
        ));

        let raw_duplicate = conn.execute(
            "INSERT INTO images (
                image_id, pixiv_id, page_index, category, local_path,
                downloaded_at, created_at, updated_at
            ) VALUES (
                'image-3', '144920810', 0, 'normal', '/tmp/other.png',
                '2026-05-21T00:00:00Z', '2026-05-21T00:00:00Z', '2026-05-21T00:00:00Z'
            )",
            [],
        );
        assert!(matches!(
            raw_duplicate,
            Err(SqliteError::SqliteFailure(error, _))
                if error.code == SqliteErrorCode::ConstraintViolation
        ));
    }

    #[test]
    fn req_img_004_req_sec_002_resolves_image_file_inside_allowed_root() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_image_file_ok_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("originals/144920810")).unwrap();
        let image_path = root.join("originals/144920810/144920810_p0.jpg");
        fs::write(&image_path, b"preview bytes").unwrap();

        let mut image = sample_image("image-1", "144920810", 0);
        image.local_path = image_path.to_string_lossy().to_string();
        repo.insert(&image).unwrap();
        let stored = repo.find_by_id("image-1").unwrap().unwrap();

        let file = resolve_image_file(&stored, std::slice::from_ref(&root)).unwrap();

        assert_eq!(file.path, image_path.canonicalize().unwrap());
        assert_eq!(file.content_type, "image/jpeg");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_img_004_req_sec_002_rejects_image_file_outside_allowed_root() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_image_file_root_{}",
            std::process::id()
        ));
        let outside = std::env::temp_dir().join(format!(
            "pixiv_platform_image_file_outside_{}.jpg",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, b"outside bytes").unwrap();

        let mut image = sample_image("image-1", "144920810", 0);
        image.local_path = outside.to_string_lossy().to_string();
        repo.insert(&image).unwrap();
        let stored = repo.find_by_id("image-1").unwrap().unwrap();

        let error = resolve_image_file(&stored, &[root.clone()]).unwrap_err();

        assert_eq!(error.code, crate::errors::ErrorCode::ValidationError);
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn req_img_007_req_sec_002_delete_image_removes_file_and_index_rows() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_image_delete_ok_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("originals/144920810")).unwrap();
        let image_path = root.join("originals/144920810/144920810_p0.jpg");
        fs::write(&image_path, b"delete bytes").unwrap();

        let mut image = sample_image("image-1", "144920810", 0);
        image.local_path = image_path.to_string_lossy().to_string();
        repo.insert(&image).unwrap();
        repo.replace_tags("image-1", &["cyan".to_owned()]).unwrap();
        repo.add_source("image-1", ImageSource::Single, None)
            .unwrap();

        let outcome =
            delete_image_file_and_index(&repo, "image-1", std::slice::from_ref(&root)).unwrap();

        assert_eq!(outcome.image_id, "image-1");
        assert!(outcome.file_deleted);
        assert!(!outcome.file_missing);
        assert!(!image_path.exists());
        assert!(repo.find_by_id("image-1").unwrap().is_none());
        assert!(repo.tags_for_image("image-1").unwrap().is_empty());
        assert!(repo.sources_for_image("image-1").unwrap().is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_img_007_delete_image_cleans_index_when_file_is_already_missing() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_image_delete_missing_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("originals/144920810")).unwrap();
        let image_path = root.join("originals/144920810/144920810_p0.jpg");

        let mut image = sample_image("image-1", "144920810", 0);
        image.local_path = image_path.to_string_lossy().to_string();
        repo.insert(&image).unwrap();

        let outcome =
            delete_image_file_and_index(&repo, "image-1", std::slice::from_ref(&root)).unwrap();

        assert!(!outcome.file_deleted);
        assert!(outcome.file_missing);
        assert!(repo.find_by_id("image-1").unwrap().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_img_007_req_sec_002_delete_image_rejects_outside_allowed_root() {
        let conn = open_in_memory().unwrap();
        let repo = ImageRepository::new(&conn);
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_image_delete_root_{}",
            std::process::id()
        ));
        let outside = std::env::temp_dir().join(format!(
            "pixiv_platform_image_delete_outside_{}.jpg",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, b"outside bytes").unwrap();

        let mut image = sample_image("image-1", "144920810", 0);
        image.local_path = outside.to_string_lossy().to_string();
        repo.insert(&image).unwrap();

        let error = delete_image_file_and_index(&repo, "image-1", &[root.clone()]).unwrap_err();

        assert_eq!(error.code, crate::errors::ErrorCode::ValidationError);
        assert!(outside.exists());
        assert!(repo.find_by_id("image-1").unwrap().is_some());
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn req_img_004_preview_url_encodes_image_id_without_local_path() {
        assert_eq!(
            preview_url_for("pixiv:144920810:p0"),
            "/api/images/pixiv%3A144920810%3Ap0/file"
        );
    }

    fn sample_image(image_id: &str, pixiv_id: &str, page_index: u32) -> NewImageRecord {
        NewImageRecord {
            image_id: image_id.to_owned(),
            pixiv_id: pixiv_id.to_owned(),
            page_index,
            author_uid: Some("999".to_owned()),
            title: Some("おでかけ".to_owned()),
            category: ImageCategory::Normal,
            local_path: format!("/tmp/{pixiv_id}_p{page_index}.png"),
            thumbnail_path: None,
            width: Some(1062),
            height: Some(1500),
            map_x: None,
            map_y: None,
            downloaded_at: "2026-05-21T00:00:00Z".to_owned(),
        }
    }
}
