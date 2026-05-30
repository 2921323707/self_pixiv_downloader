use rusqlite::{Connection, OptionalExtension, params};

use crate::errors::{AppError, ErrorCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivAccountRecord {
    pub user_uid: String,
    pub user_name: Option<String>,
    pub is_active: bool,
    pub last_verified_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivAccountSecret {
    pub user_uid: String,
    pub user_name: Option<String>,
    pub cookie_json: String,
    pub is_active: bool,
    pub last_verified_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct PixivAccountRepository<'conn> {
    conn: &'conn Connection,
}

impl<'conn> PixivAccountRepository<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    pub fn upsert_active(
        &self,
        user_uid: &str,
        user_name: Option<&str>,
        cookie: &str,
    ) -> Result<PixivAccountRecord, AppError> {
        let user_uid = user_uid.trim();
        if user_uid.is_empty() {
            return Err(AppError::validation("user_uid cannot be empty"));
        }
        if !user_uid.chars().all(|c| c.is_ascii_digit()) {
            return Err(AppError::validation("user_uid must contain only digits"));
        }
        if cookie.trim().is_empty() {
            return Err(AppError::validation("pixiv account cookie cannot be empty"));
        }

        let user_name = user_name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let cookie_json = serde_json::to_string(cookie).map_err(|error| {
            AppError::new(
                ErrorCode::InternalError,
                format!("pixiv account cookie could not be encoded: {error}"),
            )
        })?;

        self.conn.execute(
            "UPDATE pixiv_accounts SET is_active = 0 WHERE is_active = 1",
            [],
        )?;
        self.conn.execute(
            "INSERT INTO pixiv_accounts (
                user_uid, user_name, cookie_json, is_active, last_verified_at, created_at, updated_at
             )
             VALUES (?1, ?2, ?3, 1, datetime('now'), datetime('now'), datetime('now'))
             ON CONFLICT(user_uid) DO UPDATE SET
                user_name = excluded.user_name,
                cookie_json = excluded.cookie_json,
                is_active = 1,
                last_verified_at = excluded.last_verified_at,
                updated_at = excluded.updated_at",
            params![user_uid, user_name, cookie_json],
        )?;

        self.get_public(user_uid)?
            .ok_or_else(|| AppError::validation("pixiv account was not saved"))
    }

    pub fn list_public(&self) -> Result<Vec<PixivAccountRecord>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT user_uid, user_name, is_active, last_verified_at, created_at, updated_at
             FROM pixiv_accounts
             ORDER BY is_active DESC, updated_at DESC, user_uid",
        )?;
        stmt.query_map([], public_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)
    }

    pub fn get_public(&self, user_uid: &str) -> Result<Option<PixivAccountRecord>, AppError> {
        self.conn
            .query_row(
                "SELECT user_uid, user_name, is_active, last_verified_at, created_at, updated_at
                 FROM pixiv_accounts
                 WHERE user_uid = ?1",
                params![user_uid],
                public_row,
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn get_active_public(&self) -> Result<Option<PixivAccountRecord>, AppError> {
        self.conn
            .query_row(
                "SELECT user_uid, user_name, is_active, last_verified_at, created_at, updated_at
                 FROM pixiv_accounts
                 WHERE is_active = 1
                 LIMIT 1",
                [],
                public_row,
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn get_secret(&self, user_uid: &str) -> Result<Option<PixivAccountSecret>, AppError> {
        self.conn
            .query_row(
                "SELECT user_uid, user_name, cookie_json, is_active, last_verified_at, created_at, updated_at
                 FROM pixiv_accounts
                 WHERE user_uid = ?1",
                params![user_uid],
                secret_row,
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn set_active(&self, user_uid: &str) -> Result<PixivAccountSecret, AppError> {
        let Some(account) = self.get_secret(user_uid)? else {
            return Err(AppError::validation("pixiv account was not found"));
        };
        self.conn.execute(
            "UPDATE pixiv_accounts SET is_active = 0 WHERE is_active = 1",
            [],
        )?;
        self.conn.execute(
            "UPDATE pixiv_accounts SET is_active = 1, updated_at = datetime('now') WHERE user_uid = ?1",
            params![user_uid],
        )?;
        Ok(PixivAccountSecret {
            is_active: true,
            ..account
        })
    }

    pub fn delete(&self, user_uid: &str) -> Result<bool, AppError> {
        let changed = self.conn.execute(
            "DELETE FROM pixiv_accounts WHERE user_uid = ?1",
            params![user_uid],
        )?;
        Ok(changed > 0)
    }
}

fn public_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PixivAccountRecord> {
    Ok(PixivAccountRecord {
        user_uid: row.get(0)?,
        user_name: row.get(1)?,
        is_active: row.get::<_, i64>(2)? == 1,
        last_verified_at: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn secret_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PixivAccountSecret> {
    Ok(PixivAccountSecret {
        user_uid: row.get(0)?,
        user_name: row.get(1)?,
        cookie_json: row.get(2)?,
        is_active: row.get::<_, i64>(3)? == 1,
        last_verified_at: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
