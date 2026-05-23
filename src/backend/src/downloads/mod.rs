use std::path::PathBuf;

use crate::domain::{DownloadItemStatus, DownloadOutcome, DownloadRequest, PixivPage, PixivWork};
use crate::errors::{AppError, ErrorCode};
use crate::images::{ImageRecord, ImageRepository, NewImageRecord};
use crate::pixiv::PixivClient;
use crate::storage::StoragePlanner;

pub struct DownloadRepositoryContext<'conn> {
    images: ImageRepository<'conn>,
    source_task_id: Option<String>,
}

impl<'conn> DownloadRepositoryContext<'conn> {
    pub fn new(conn: &'conn rusqlite::Connection) -> Self {
        Self {
            images: ImageRepository::new(conn),
            source_task_id: None,
        }
    }

    pub fn for_task(conn: &'conn rusqlite::Connection, task_id: impl Into<String>) -> Self {
        Self {
            images: ImageRepository::new(conn),
            source_task_id: Some(task_id.into()),
        }
    }
}

pub fn download_single(
    request: &DownloadRequest,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
) -> Result<DownloadOutcome, AppError> {
    validate_request(request)?;

    let work = pixiv.fetch_work(&request.pixiv_id)?;
    if !request.r18_policy.allows(work.category) {
        return Ok(outcome_for_policy_skip(
            &work,
            request.page_index.unwrap_or(0),
        ));
    }

    let page = select_page(&work, request.page_index)?;
    let final_path = storage.original_path(
        &work.pixiv_id,
        page.page_index,
        page.extension
            .as_deref()
            .or_else(|| extension_from_url(&page.original_url)),
    )?;

    if final_path.exists() {
        return Ok(outcome_for_existing_file(
            &work,
            page.page_index,
            final_path,
        ));
    }

    let bytes = pixiv.download_image(&page.original_url)?;
    storage.write_atomic(&final_path, &bytes)?;

    Ok(DownloadOutcome {
        pixiv_id: work.pixiv_id.clone(),
        page_index: page.page_index,
        status: DownloadItemStatus::Saved,
        local_path: Some(final_path),
        metadata: Some(work),
    })
}

pub fn download_single_with_db(
    request: &DownloadRequest,
    pixiv: &dyn PixivClient,
    storage: &StoragePlanner,
    repositories: &DownloadRepositoryContext<'_>,
) -> Result<DownloadOutcome, AppError> {
    validate_request(request)?;

    let page_index = request.page_index.unwrap_or(0);
    if let Some(existing) = repositories
        .images
        .find_by_pixiv_page(&request.pixiv_id, page_index)?
    {
        if !request.r18_policy.allows(existing.category) {
            return Ok(outcome_for_indexed_policy_skip(&existing));
        }

        let indexed_path = PathBuf::from(&existing.local_path);
        if indexed_path.exists() {
            repositories.images.add_source(
                &existing.image_id,
                request.source,
                repositories.source_task_id.as_deref(),
            )?;
            return Ok(outcome_for_indexed_duplicate(&existing, indexed_path));
        }

        let work = pixiv.fetch_work(&request.pixiv_id)?;
        if !request.r18_policy.allows(work.category) {
            return Ok(outcome_for_policy_skip(&work, page_index));
        }

        let page = select_page(&work, request.page_index)?;
        let final_path = planned_original_path(storage, &work, &page)?;
        if !final_path.exists() {
            let bytes = pixiv.download_image(&page.original_url)?;
            storage.write_atomic(&final_path, &bytes)?;
        }

        index_downloaded_page(
            repositories,
            Some(existing.image_id),
            request,
            &work,
            &page,
            final_path.clone(),
            IndexMode::UpdateExisting,
        )?;

        return Ok(DownloadOutcome {
            pixiv_id: work.pixiv_id.clone(),
            page_index: page.page_index,
            status: DownloadItemStatus::Saved,
            local_path: Some(final_path),
            metadata: Some(work),
        });
    }

    let work = pixiv.fetch_work(&request.pixiv_id)?;
    if !request.r18_policy.allows(work.category) {
        return Ok(outcome_for_policy_skip(&work, page_index));
    }

    let page = select_page(&work, request.page_index)?;
    let final_path = planned_original_path(storage, &work, &page)?;
    if final_path.exists() {
        index_downloaded_page(
            repositories,
            None,
            request,
            &work,
            &page,
            final_path.clone(),
            IndexMode::InsertNew,
        )?;
        return Ok(outcome_for_existing_file(
            &work,
            page.page_index,
            final_path,
        ));
    }

    let bytes = pixiv.download_image(&page.original_url)?;
    storage.write_atomic(&final_path, &bytes)?;
    index_downloaded_page(
        repositories,
        None,
        request,
        &work,
        &page,
        final_path.clone(),
        IndexMode::InsertNew,
    )?;

    Ok(DownloadOutcome {
        pixiv_id: work.pixiv_id.clone(),
        page_index: page.page_index,
        status: DownloadItemStatus::Saved,
        local_path: Some(final_path),
        metadata: Some(work),
    })
}

fn validate_request(request: &DownloadRequest) -> Result<(), AppError> {
    if request.pixiv_id.trim().is_empty() {
        return Err(AppError::validation("pixiv_id cannot be empty"));
    }
    if !request.pixiv_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::validation("pixiv_id must contain only digits"));
    }
    Ok(())
}

fn planned_original_path(
    storage: &StoragePlanner,
    work: &PixivWork,
    page: &PixivPage,
) -> Result<PathBuf, AppError> {
    storage.original_path(
        &work.pixiv_id,
        page.page_index,
        page.extension
            .as_deref()
            .or_else(|| extension_from_url(&page.original_url)),
    )
}

fn select_page(work: &PixivWork, page_index: Option<u32>) -> Result<PixivPage, AppError> {
    let wanted = page_index.unwrap_or(0);
    work.pages
        .iter()
        .find(|page| page.page_index == wanted)
        .cloned()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::PixivNotFound,
                format!("Pixiv work {} has no page index {}", work.pixiv_id, wanted),
            )
        })
}

enum IndexMode {
    InsertNew,
    UpdateExisting,
}

fn index_downloaded_page(
    repositories: &DownloadRepositoryContext<'_>,
    image_id: Option<String>,
    request: &DownloadRequest,
    work: &PixivWork,
    page: &PixivPage,
    final_path: PathBuf,
    mode: IndexMode,
) -> Result<(), AppError> {
    let record = NewImageRecord {
        image_id: image_id
            .unwrap_or_else(|| image_id_for_pixiv_page(&work.pixiv_id, page.page_index)),
        pixiv_id: work.pixiv_id.clone(),
        page_index: page.page_index,
        author_uid: work.author_uid.clone(),
        title: work.title.clone(),
        category: work.category,
        local_path: final_path.to_string_lossy().into_owned(),
        thumbnail_path: None,
        width: page.width,
        height: page.height,
        map_x: None,
        map_y: None,
        downloaded_at: repositories.images.current_timestamp()?,
    };

    match mode {
        IndexMode::InsertNew => repositories.images.insert(&record)?,
        IndexMode::UpdateExisting => repositories.images.update_from_download(&record)?,
    }

    repositories
        .images
        .replace_tags(&record.image_id, &work.tags)?;
    repositories.images.add_source(
        &record.image_id,
        request.source,
        repositories.source_task_id.as_deref(),
    )?;
    Ok(())
}

fn image_id_for_pixiv_page(pixiv_id: &str, page_index: u32) -> String {
    format!("pixiv:{pixiv_id}:p{page_index}")
}

fn extension_from_url(url: &str) -> Option<&str> {
    url.rsplit_once('.')
        .map(|(_, ext)| ext)
        .and_then(|ext| ext.split('?').next())
        .filter(|ext| !ext.is_empty())
}

fn outcome_for_policy_skip(work: &PixivWork, page_index: u32) -> DownloadOutcome {
    DownloadOutcome {
        pixiv_id: work.pixiv_id.clone(),
        page_index,
        status: DownloadItemStatus::SkippedByPolicy,
        local_path: None,
        metadata: Some(work.clone()),
    }
}

fn outcome_for_indexed_policy_skip(image: &ImageRecord) -> DownloadOutcome {
    DownloadOutcome {
        pixiv_id: image.pixiv_id.clone(),
        page_index: image.page_index,
        status: DownloadItemStatus::SkippedByPolicy,
        local_path: None,
        metadata: None,
    }
}

fn outcome_for_indexed_duplicate(image: &ImageRecord, path: PathBuf) -> DownloadOutcome {
    DownloadOutcome {
        pixiv_id: image.pixiv_id.clone(),
        page_index: image.page_index,
        status: DownloadItemStatus::SkippedDuplicate,
        local_path: Some(path),
        metadata: None,
    }
}

fn outcome_for_existing_file(work: &PixivWork, page_index: u32, path: PathBuf) -> DownloadOutcome {
    DownloadOutcome {
        pixiv_id: work.pixiv_id.clone(),
        page_index,
        status: DownloadItemStatus::SkippedDuplicate,
        local_path: Some(path),
        metadata: Some(work.clone()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::domain::{
        DownloadItemStatus, DownloadRequest, ImageCategory, ImageSource, PixivPage, PixivWork,
        R18Policy,
    };
    use crate::downloads::{DownloadRepositoryContext, download_single, download_single_with_db};
    use crate::images::{ImageRepository, NewImageRecord};
    use crate::pixiv::mock::MockPixivClient;
    use crate::storage::StoragePlanner;

    fn test_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "pixiv_platform_backend_{name}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn sample_work(category: ImageCategory) -> PixivWork {
        PixivWork {
            pixiv_id: "123456".to_owned(),
            title: Some("mock work".to_owned()),
            author_uid: Some("9988".to_owned()),
            author_name: Some("mock author".to_owned()),
            tags: vec!["blue hair".to_owned()],
            category,
            pages: vec![PixivPage {
                page_index: 0,
                original_url: "https://i.pximg.net/img-original/mock/123456_p0.jpg".to_owned(),
                width: Some(1200),
                height: Some(1800),
                extension: Some("jpg".to_owned()),
            }],
        }
    }

    fn request(policy: R18Policy) -> DownloadRequest {
        DownloadRequest {
            pixiv_id: "123456".to_owned(),
            page_index: Some(0),
            source: ImageSource::Single,
            r18_policy: policy,
        }
    }

    fn expected_path(storage: &StoragePlanner) -> std::path::PathBuf {
        storage.original_path("123456", 0, Some("jpg")).unwrap()
    }

    fn preindexed_image(local_path: std::path::PathBuf) -> NewImageRecord {
        NewImageRecord {
            image_id: "existing-image".to_owned(),
            pixiv_id: "123456".to_owned(),
            page_index: 0,
            author_uid: Some("old-author".to_owned()),
            title: Some("old title".to_owned()),
            category: ImageCategory::Normal,
            local_path: local_path.to_string_lossy().into_owned(),
            thumbnail_path: None,
            width: Some(100),
            height: Some(200),
            map_x: None,
            map_y: None,
            downloaded_at: "2026-05-21T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn req_dl_001_downloads_single_work_with_mock_pixiv() {
        let root = test_root("single_success");
        let work = sample_work(ImageCategory::Normal);
        let client = MockPixivClient::default().with_work(work).with_image(
            "https://i.pximg.net/img-original/mock/123456_p0.jpg",
            b"fake image bytes".to_vec(),
        );
        let storage = StoragePlanner::new(&root);

        let outcome = download_single(&request(R18Policy::Exclude), &client, &storage).unwrap();

        assert_eq!(outcome.status, DownloadItemStatus::Saved);
        let local_path = outcome.local_path.unwrap();
        assert!(local_path.exists());
        assert_eq!(fs::read(local_path).unwrap(), b"fake image bytes");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_006_skips_existing_file_as_duplicate() {
        let root = test_root("duplicate");
        let work = sample_work(ImageCategory::Normal);
        let client = MockPixivClient::default().with_work(work).with_image(
            "https://i.pximg.net/img-original/mock/123456_p0.jpg",
            b"fake image bytes".to_vec(),
        );
        let storage = StoragePlanner::new(&root);

        let first = download_single(&request(R18Policy::Exclude), &client, &storage).unwrap();
        let second = download_single(&request(R18Policy::Exclude), &client, &storage).unwrap();

        assert_eq!(first.status, DownloadItemStatus::Saved);
        assert_eq!(second.status, DownloadItemStatus::SkippedDuplicate);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_001_req_img_001_db_aware_first_download_indexes_file_tags_and_source() {
        let root = test_root("db_first_download");
        let conn = crate::db::open_in_memory().unwrap();
        let work = sample_work(ImageCategory::Normal);
        let client = MockPixivClient::default().with_work(work).with_image(
            "https://i.pximg.net/img-original/mock/123456_p0.jpg",
            b"fake image bytes".to_vec(),
        );
        let storage = StoragePlanner::new(&root);

        let outcome = {
            let repositories = DownloadRepositoryContext::new(&conn);
            download_single_with_db(
                &request(R18Policy::Exclude),
                &client,
                &storage,
                &repositories,
            )
            .unwrap()
        };

        assert_eq!(outcome.status, DownloadItemStatus::Saved);
        let local_path = outcome.local_path.unwrap();
        assert_eq!(fs::read(&local_path).unwrap(), b"fake image bytes");

        let repo = ImageRepository::new(&conn);
        let image = repo.find_by_pixiv_page("123456", 0).unwrap().unwrap();
        assert_eq!(image.local_path, local_path.to_string_lossy());
        assert_eq!(image.width, Some(1200));
        assert_eq!(image.height, Some(1800));
        assert_eq!(
            repo.tags_for_image(&image.image_id).unwrap(),
            vec!["blue hair"]
        );
        let sources = repo.sources_for_image(&image.image_id).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source, ImageSource::Single);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_006_db_duplicate_skip_avoids_image_download_and_records_source() {
        let root = test_root("db_duplicate_skip");
        let conn = crate::db::open_in_memory().unwrap();
        let storage = StoragePlanner::new(&root);
        let local_path = expected_path(&storage);
        fs::create_dir_all(local_path.parent().unwrap()).unwrap();
        fs::write(&local_path, b"already here").unwrap();
        let repo = ImageRepository::new(&conn);
        repo.insert(&preindexed_image(local_path.clone())).unwrap();
        let client = MockPixivClient::default();

        let outcome = {
            let repositories = DownloadRepositoryContext::new(&conn);
            download_single_with_db(
                &request(R18Policy::Exclude),
                &client,
                &storage,
                &repositories,
            )
            .unwrap()
        };

        assert_eq!(outcome.status, DownloadItemStatus::SkippedDuplicate);
        assert_eq!(outcome.local_path.as_deref(), Some(local_path.as_path()));
        let sources = repo.sources_for_image("existing-image").unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source, ImageSource::Single);
        assert_eq!(fs::read(local_path).unwrap(), b"already here");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_006_missing_file_repair_redownloads_and_refreshes_index() {
        let root = test_root("db_missing_file_repair");
        let conn = crate::db::open_in_memory().unwrap();
        let storage = StoragePlanner::new(&root);
        let missing_path = expected_path(&storage);
        let repo = ImageRepository::new(&conn);
        repo.insert(&preindexed_image(missing_path.clone()))
            .unwrap();
        let work = sample_work(ImageCategory::Normal);
        let client = MockPixivClient::default().with_work(work).with_image(
            "https://i.pximg.net/img-original/mock/123456_p0.jpg",
            b"repaired image bytes".to_vec(),
        );

        let outcome = {
            let repositories = DownloadRepositoryContext::new(&conn);
            download_single_with_db(
                &request(R18Policy::Exclude),
                &client,
                &storage,
                &repositories,
            )
            .unwrap()
        };

        assert_eq!(outcome.status, DownloadItemStatus::Saved);
        assert_eq!(fs::read(&missing_path).unwrap(), b"repaired image bytes");
        let image = repo.find_by_pixiv_page("123456", 0).unwrap().unwrap();
        assert_eq!(image.image_id, "existing-image");
        assert_eq!(image.title.as_deref(), Some("mock work"));
        assert_eq!(image.author_uid.as_deref(), Some("9988"));
        assert_eq!(
            repo.tags_for_image("existing-image").unwrap(),
            vec!["blue hair"]
        );
        assert_eq!(repo.sources_for_image("existing-image").unwrap().len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_006_existing_file_indexing_inserts_db_without_downloading_bytes() {
        let root = test_root("existing_file_indexing");
        let conn = crate::db::open_in_memory().unwrap();
        let storage = StoragePlanner::new(&root);
        let local_path = expected_path(&storage);
        fs::create_dir_all(local_path.parent().unwrap()).unwrap();
        fs::write(&local_path, b"existing file bytes").unwrap();
        let work = sample_work(ImageCategory::Normal);
        let client = MockPixivClient::default().with_work(work);

        let outcome = {
            let repositories = DownloadRepositoryContext::new(&conn);
            download_single_with_db(
                &request(R18Policy::Exclude),
                &client,
                &storage,
                &repositories,
            )
            .unwrap()
        };

        assert_eq!(outcome.status, DownloadItemStatus::SkippedDuplicate);
        assert_eq!(fs::read(&local_path).unwrap(), b"existing file bytes");
        let repo = ImageRepository::new(&conn);
        let image = repo.find_by_pixiv_page("123456", 0).unwrap().unwrap();
        assert_eq!(image.local_path, local_path.to_string_lossy());
        assert_eq!(
            image.image_id,
            crate::downloads::image_id_for_pixiv_page("123456", 0)
        );
        assert_eq!(
            repo.tags_for_image(&image.image_id).unwrap(),
            vec!["blue hair"]
        );
        assert_eq!(repo.sources_for_image(&image.image_id).unwrap().len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_cfg_005_skips_r18_when_policy_excludes_it() {
        let root = test_root("r18_skip");
        let work = sample_work(ImageCategory::R18);
        let client = MockPixivClient::default().with_work(work);
        let storage = StoragePlanner::new(&root);

        let outcome = download_single(&request(R18Policy::Exclude), &client, &storage).unwrap();

        assert_eq!(outcome.status, DownloadItemStatus::SkippedByPolicy);
        assert!(outcome.local_path.is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn req_dl_001_rejects_invalid_pixiv_id() {
        let root = test_root("invalid_id");
        let client = MockPixivClient::default();
        let storage = StoragePlanner::new(&root);
        let bad_request = DownloadRequest {
            pixiv_id: "../123".to_owned(),
            page_index: Some(0),
            source: ImageSource::Single,
            r18_policy: R18Policy::Exclude,
        };

        let error = download_single(&bad_request, &client, &storage).unwrap_err();

        assert_eq!(error.code.as_str(), "VALIDATION_ERROR");
        let _ = fs::remove_dir_all(root);
    }
}
