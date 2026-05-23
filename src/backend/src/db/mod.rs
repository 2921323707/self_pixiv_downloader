use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::errors::AppError;

const MIGRATIONS: &[(&str, &str)] =
    &[("0001_init", include_str!("../../migrations/0001_init.sql"))];

pub fn open(path: impl AsRef<Path>) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    initialize(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    initialize(&conn)?;
    Ok(conn)
}

pub fn initialize(conn: &Connection) -> Result<(), AppError> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    run_migrations(conn)?;
    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            migration_id TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    for (migration_id, sql) in MIGRATIONS {
        if migration_applied(conn, migration_id)? {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (migration_id, applied_at)
             VALUES (?1, datetime('now'))",
            params![migration_id],
        )?;
        tx.commit()?;
    }

    Ok(())
}

fn migration_applied(conn: &Connection, migration_id: &str) -> Result<bool, AppError> {
    let found = conn
        .query_row(
            "SELECT 1 FROM schema_migrations WHERE migration_id = ?1",
            params![migration_id],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use rusqlite::{Error as SqliteError, ErrorCode as SqliteErrorCode, OptionalExtension, params};

    use crate::db::open_in_memory;

    #[test]
    fn req_img_001_migration_creates_core_image_tables() {
        let conn = open_in_memory().unwrap();

        assert!(table_exists(&conn, "images"));
        assert!(table_exists(&conn, "image_tags"));
        assert!(table_exists(&conn, "image_sources"));
    }

    #[test]
    fn req_task_002_migration_creates_task_traceability_tables() {
        let conn = open_in_memory().unwrap();

        assert!(table_exists(&conn, "tasks"));
        assert!(table_exists(&conn, "task_logs"));
        assert!(table_exists(&conn, "task_items"));
    }

    #[test]
    fn req_dl_006_migration_enforces_unique_pixiv_page_identity() {
        let conn = open_in_memory().unwrap();
        insert_image(&conn, "image-1", "144920810", 0);

        let duplicate = conn.execute(
            "INSERT INTO images (
                image_id, pixiv_id, page_index, category, local_path,
                downloaded_at, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 'normal', '/tmp/duplicate.png', ?4, ?4, ?4)",
            params!["image-2", "144920810", 0, "2026-05-21T00:00:00Z"],
        );

        assert!(matches!(
            duplicate,
            Err(SqliteError::SqliteFailure(error, _))
                if error.code == SqliteErrorCode::ConstraintViolation
        ));
    }

    #[test]
    fn req_img_001_migration_enforces_tag_foreign_key() {
        let conn = open_in_memory().unwrap();

        let result = conn.execute(
            "INSERT INTO image_tags (image_id, tag, created_at)
             VALUES ('missing-image', 'blue hair', '2026-05-21T00:00:00Z')",
            [],
        );

        assert!(matches!(result, Err(SqliteError::SqliteFailure(_, _))));
    }

    #[test]
    fn req_cfg_006_migration_sets_cyan_studio_default_theme() {
        let conn = open_in_memory().unwrap();

        let theme: String = conn
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'theme_id'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(theme, "\"cyan-studio\"");
    }

    fn table_exists(conn: &rusqlite::Connection, table_name: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![table_name],
            |_| Ok(()),
        )
        .optional()
        .unwrap()
        .is_some()
    }

    fn insert_image(conn: &rusqlite::Connection, image_id: &str, pixiv_id: &str, page_index: i64) {
        conn.execute(
            "INSERT INTO images (
                image_id, pixiv_id, page_index, category, local_path,
                downloaded_at, created_at, updated_at
             ) VALUES (?1, ?2, ?3, 'normal', '/tmp/mock.png', ?4, ?4, ?4)",
            params![image_id, pixiv_id, page_index, "2026-05-21T00:00:00Z"],
        )
        .unwrap();
    }
}
