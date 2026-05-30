use std::fs;

use rusqlite::{Connection, Result};
use tauri::Manager;

pub fn connect_db(app_handle: &tauri::AppHandle) -> Result<Connection, String> {
    let mut app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }

    app_dir.push("memoake.db");

    Connection::open(app_dir).map_err(|e| e.to_string())
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
