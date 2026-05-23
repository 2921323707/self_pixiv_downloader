use std::env;
use std::path::PathBuf;

use pixiv_platform_backend::domain::{DownloadRequest, ImageSource, R18Policy};
use pixiv_platform_backend::images::ImageRepository;
use pixiv_platform_backend::pixiv::http::PixivHttpClient;
use pixiv_platform_backend::storage::StoragePlanner;
use pixiv_platform_backend::tasks::{TaskRepository, run_single_download_task};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}: {}", error.code.as_str(), error.message);
        std::process::exit(1);
    }
}

fn run() -> Result<(), pixiv_platform_backend::errors::AppError> {
    let cookie = env::var("PIXIV_PHPSESSID").map_err(|_| {
        pixiv_platform_backend::errors::AppError::new(
            pixiv_platform_backend::errors::ErrorCode::MissingPixivCookie,
            "PIXIV_PHPSESSID is required",
        )
    })?;
    let pixiv_id = env::var("PIXIV_TEST_WORK_ID").unwrap_or_else(|_| "144920810".to_owned());
    let download_root = env::var("PIXIV_TEST_DOWNLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir().join("pixiv_platform_live"));
    let db_path = env::var("PIXIV_TEST_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| download_root.join("pixiv_platform.sqlite3"));

    let client = PixivHttpClient::new(cookie)?;
    let storage = StoragePlanner::new(&download_root);
    let conn = pixiv_platform_backend::db::open(&db_path)?;
    let request = DownloadRequest {
        pixiv_id,
        page_index: Some(0),
        source: ImageSource::Single,
        r18_policy: R18Policy::IncludeBlurred,
    };

    let outcome = run_single_download_task(&request, &client, &storage, &conn)?;
    let task_repo = TaskRepository::new(&conn);
    let task = task_repo.find_task(&outcome.task_id)?.ok_or_else(|| {
        pixiv_platform_backend::errors::AppError::new(
            pixiv_platform_backend::errors::ErrorCode::InternalError,
            "task was not persisted",
        )
    })?;

    println!("status={:?}", outcome.download_status);
    println!("task_id={}", outcome.task_id);
    println!("task_status={}", task.status.as_str());
    println!("task_progress_done={}", task.progress_done);
    println!("task_progress_failed={}", task.progress_failed);
    println!("pixiv_id={}", request.pixiv_id);
    println!("page_index={}", request.page_index.unwrap_or(0));
    println!("download_root={}", download_root.display());
    println!("db_path={}", db_path.display());

    let repo = ImageRepository::new(&conn);
    if let Some(image) =
        repo.find_by_pixiv_page(&request.pixiv_id, request.page_index.unwrap_or(0))?
    {
        println!("db_image_id={}", image.image_id);
        println!("db_local_path={}", image.local_path);
        println!("local_path={}", image.local_path);
        println!("db_tags={}", repo.tags_for_image(&image.image_id)?.len());
        println!(
            "db_sources={}",
            repo.sources_for_image(&image.image_id)?.len()
        );
    }
    println!(
        "task_items={}",
        task_repo.items_for_task(&outcome.task_id)?.len()
    );
    println!(
        "task_logs={}",
        task_repo.logs_for_task(&outcome.task_id)?.len()
    );

    Ok(())
}
