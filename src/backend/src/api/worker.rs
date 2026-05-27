use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::db;
use crate::errors::{AppError, ErrorCode};
use crate::storage::StoragePlanner;
use crate::tasks::{TaskRepository, execute_queued_task};

use super::runtime::{prepare_db_path, resolve_runtime_settings};
use super::{PixivClientFactory, QueuedTask};

pub(crate) fn spawn_worker(
    mut task_receiver: mpsc::Receiver<QueuedTask>,
    db_path: PathBuf,
    download_root: PathBuf,
    pixiv_client_factory: Arc<dyn PixivClientFactory>,
) {
    tokio::spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            let task_id = task.task_id;
            let db_path_for_job = db_path.clone();
            let download_root_for_job = download_root.clone();
            let factory_for_job = Arc::clone(&pixiv_client_factory);
            let task_id_for_job = task_id.clone();

            let result = tokio::task::spawn_blocking(move || {
                prepare_db_path(&db_path_for_job)?;
                let conn = db::open(&db_path_for_job)?;
                let runtime = resolve_runtime_settings(&conn, &download_root_for_job)?;
                fs::create_dir_all(&runtime.download_root)?;
                let storage = StoragePlanner::new(runtime.download_root);
                let pixiv = factory_for_job.create_with_cookie(runtime.pixiv_cookie.as_deref())?;
                execute_queued_task(&task_id_for_job, pixiv.as_ref(), &storage, &conn)
            })
            .await;

            match result {
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    let _ =
                        mark_task_failed(&db_path, &task_id, error.code.as_str(), &error.message);
                }
                Err(error) => {
                    let _ = mark_task_failed(
                        &db_path,
                        &task_id,
                        ErrorCode::InternalError.as_str(),
                        &error.to_string(),
                    );
                }
            }
        }
    });
}

pub(crate) fn mark_task_failed(
    db_path: &PathBuf,
    task_id: &str,
    error_code: &str,
    error_message: &str,
) -> Result<(), AppError> {
    let conn = db::open(db_path)?;
    let repo = TaskRepository::new(&conn);
    if repo
        .find_task(task_id)?
        .is_some_and(|task| task.status.is_terminal())
    {
        return Ok(());
    }
    repo.fail_task(task_id, error_code, error_message)
}
