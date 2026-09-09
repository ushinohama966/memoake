use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{eprintln, sync::Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

pub mod db;

struct WindowState {
    is_visible: Mutex<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Memo {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn watch_db_file(app_handle: tauri::AppHandle) -> Result<(), String> {
    let db_path = db::get_default_db_path()?;
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher =
        notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
            Ok(event) => {
                if let EventKind::Modify(_) = event.kind {
                    if let Err(err) = tx.send(()) {
                        eprintln!("Failed to send event: {}", err);
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        })
        .map_err(|e| e.to_string())?;

    watcher
        .watch(&db_path, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    std::thread::spawn(move || {
        while let Ok(_) = rx.recv() {
            // if let Some(window) = app_handle.get_window("main") {
            if let Some(window) = app_handle.get_webview_window("main") {
                window.emit("db-changed", {}).unwrap_or_else(|err| {
                    eprintln!("Failed to emit event: {}", err);
                });
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn get_all_memo(_app_handle: tauri::AppHandle) -> Result<Vec<Memo>, String> {
    let conn = db::connect_db()?;
    let memos = db::get_all_memo(conn)?;
    Ok(memos)
}

#[tauri::command]
fn create_memo(content: &str, _app_handle: tauri::AppHandle) -> Result<Memo, String> {
    let conn = db::connect_db()?;
    let new_memo = db::create_memo(conn, content)?;
    Ok(new_memo)
}

#[tauri::command]
fn update_memo(id: i64, content: &str, _app_handle: tauri::AppHandle) -> Result<Memo, String> {
    let conn = db::connect_db()?;
    let updated_memo = db::update_memo(conn, id, content)?;

    Ok(updated_memo)
}

#[tauri::command]
fn delete_memo(id: i64, _app_handle: tauri::AppHandle) -> Result<(), String> {
    let conn = db::connect_db()?;
    db::delete_memo(conn, id)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    let toggle_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyJ);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(WindowState {
            is_visible: Mutex::new(false),
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app_handle, shortcut, event| {
                    if shortcut == &toggle_shortcut {
                        match event.state() {
                            ShortcutState::Pressed => {
                                use tauri::Manager;

                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let state = app_handle.state::<WindowState>();
                                    let mut is_visible_guard = state.is_visible.lock().unwrap();
                                    if *is_visible_guard {
                                        let _ = window.set_always_on_top(false);
                                        let _ = window.hide();
                                        *is_visible_guard = false;
                                    } else {
                                        if let Ok(_monitors) = window.available_monitors() {
                                            if let Ok(Some(monitor)) = window.current_monitor() {
                                                use tauri::{
                                                    PhysicalPosition, PhysicalSize, Position, Size,
                                                };

                                                let monitor_pos = monitor.position();
                                                let monitor_size = monitor.size();

                                                let window_width = monitor_size.width / 3 * 2;
                                                let window_height = monitor_size.height / 5 * 2;

                                                let _ = window.set_size(Size::Physical(
                                                    PhysicalSize::new(window_width, window_height),
                                                ));

                                                let target_x = monitor_pos.x
                                                    + ((monitor_size.width as i32
                                                        - window_width as i32)
                                                        / 2);
                                                let target_y = monitor_pos.y;

                                                let _ = window.set_position(Position::Physical(
                                                    PhysicalPosition::new(target_x, target_y),
                                                ));

                                                let _ = window.show();
                                                let _ = window.unminimize();
                                                let _ = window.set_always_on_top(true);

                                                let _ = window.set_focus();

                                                *is_visible_guard = true;
                                            }
                                        }
                                    }
                                }
                            }
                            ShortcutState::Released => {}
                        }
                    }
                })
                .build(),
        )
        .setup(move |app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let _ = app.global_shortcut().register(toggle_shortcut);
                if let Err(err_msg) = db::init_database() {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        err_msg,
                    )));
                };
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(err_msg) = watch_db_file(handle) {
                        eprintln!("Failed to start db file watcher: {}", err_msg);
                    }
                });
                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_all_memo,
            create_memo,
            update_memo,
            delete_memo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
