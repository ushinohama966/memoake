use std::{fs, path::Path};

use rusqlite::{Connection, Result};
use tauri::Manager;

use crate::Memo;

pub fn connect_db(app_handle: &tauri::AppHandle) -> Result<Connection, String> {
    let mut app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }

    app_dir.push("memoake.db");
    connect_db_at_path(app_dir.as_path())
}

pub fn connect_db_at_path(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    Connection::open(path).map_err(|e| e.to_string())
}

pub fn init_database(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let conn = connect_db(app_handle).map_err(|e| e.to_string())?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (DATETIME('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (DATETIME('now', 'localtime'))
        );

        CREATE TRIGGER IF NOT EXISTS update_memo_modtime
        AFTER UPDATE ON memo
        BEGIN
            UPDATE memo SET updated_at = DATETIME('now', 'localtime') WHERE id = OLD.id;
        END;
        ",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn init_database_at_path(path: &Path) -> Result<(), String> {
    let conn = connect_db_at_path(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (DATETIME('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (DATETIME('now', 'localtime'))
        );

        CREATE TRIGGER IF NOT EXISTS update_memo_modtime
        AFTER UPDATE ON memo
        BEGIN
            UPDATE memo SET updated_at = DATETIME('now', 'localtime') WHERE id = OLD.id;
        END;
        ",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_all_memo(conn: Connection) -> Result<Vec<Memo>, String> {
    let mut stmt_m = conn
        .prepare("SELECT id, content, created_at, updated_at FROM memo")
        .map_err(|e| e.to_string())?;

    let memo_iter = stmt_m
        .query_map([], |row| {
            Ok(Memo {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let memos: Result<Vec<Memo>, _> = memo_iter.collect();
    let memos = memos.map_err(|e| e.to_string())?;

    Ok(memos)
}

pub fn create_memo(conn: Connection, content: &str) -> Result<Memo, String> {
    let mut stmt = conn
        .prepare(
            "INSERT INTO memo (content) VALUES (?1) RETURNING id, content, created_at, updated_at",
        )
        .map_err(|e| e.to_string())?;

    let new_memo = stmt
        .query_row([content], |row| {
            Ok(Memo {
                id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(new_memo)
}
