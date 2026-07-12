use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;
use tauri::Emitter;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(desktop)]
use tauri::menu::{MenuBuilder, SubmenuBuilder, MenuItemBuilder};
#[cfg(desktop)]
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

/// 构建应用菜单，fullscreen_label 根据全屏状态传入 "进入全屏" 或 "退出全屏"
#[cfg(desktop)]
fn build_menu(app: &tauri::AppHandle, fullscreen_label: &str) -> tauri::Result<()> {
    let handle = app;

    let playback_menu = SubmenuBuilder::new(handle, "播放")
        .item(
            &MenuItemBuilder::with_id("play_pause", "播放 / 暂停")
                .accelerator("Space")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("next", "下一首")
                .accelerator("CmdOrCtrl+Right")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("prev", "上一首")
                .accelerator("CmdOrCtrl+Left")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("shuffle", "随机播放").build(handle)?)
        .item(&MenuItemBuilder::with_id("repeat", "循环模式").build(handle)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("vol_up", "音量增大")
                .accelerator("CmdOrCtrl+Up")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("vol_down", "音量减小")
                .accelerator("CmdOrCtrl+Down")
                .build(handle)?,
        )
        .build()?;

    let playlist_menu = SubmenuBuilder::new(handle, "播放列表")
        .item(
            &MenuItemBuilder::with_id("import", "导入音频…")
                .accelerator("CmdOrCtrl+O")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("favorite", "收藏当前曲目")
                .accelerator("CmdOrCtrl+F")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("clear", "清空播放列表").build(handle)?)
        .build()?;

    let window_menu = SubmenuBuilder::new(handle, "窗口")
        .item(
            &MenuItemBuilder::with_id("minimize", "最小化")
                .accelerator("CmdOrCtrl+M")
                .build(handle)?,
        )
        .item(&MenuItemBuilder::with_id("show", "显示窗口").build(handle)?)
        .separator()
        .item(&MenuItemBuilder::with_id("zoom", "缩放").build(handle)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("toggle_fullscreen", fullscreen_label)
                .accelerator("CmdOrCtrl+Ctrl+F")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("toggle_ontop", "窗口置顶").build(handle)?)
        .build()?;

    let help_menu = SubmenuBuilder::new(handle, "帮助")
        .item(&MenuItemBuilder::with_id("about", "关于 Hanono").build(handle)?)
        .build()?;

    let menu = MenuBuilder::new(handle)
        .item(&playback_menu)
        .item(&playlist_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    app.set_menu(menu)?;
    Ok(())
}

struct DbState {
    conn: Mutex<Connection>,
    app_data_dir: PathBuf,
}

fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         CREATE TABLE IF NOT EXISTS state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    ).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_favorites(state: tauri::State<DbState>, data: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO state (key, value) VALUES ('favorites', ?1)",
        rusqlite::params![data],
    ).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
fn load_favorites(state: tauri::State<DbState>) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM state WHERE key = 'favorites'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_playlist(state: tauri::State<DbState>, data: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let len = data.len();
    let preview: String = data.chars().take(300).collect();
    conn.execute(
        "INSERT OR REPLACE INTO state (key, value) VALUES ('playlist', ?1)",
        rusqlite::params![data],
    ).map_err(|e| e.to_string())?;
    // Force flush to disk
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| e.to_string())?;
    eprintln!("[db] playlist SAVED ({} bytes) preview: {}", len, preview);
    Ok(true)
}

#[tauri::command]
fn load_playlist(state: tauri::State<DbState>) -> Result<String, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM state WHERE key = 'playlist'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(v) => {
            let preview: String = v.chars().take(300).collect();
            eprintln!("[db] playlist LOADED ({} bytes) preview: {}", v.len(), preview);
            Ok(v)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            eprintln!("[db] no saved playlist found");
            Ok(String::new())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Copy an audio file into the app's persistent data directory.
#[tauri::command]
fn copy_file_to_data(state: tauri::State<DbState>, source: String) -> Result<String, String> {
    let src = PathBuf::from(&source);
    let filename = src
        .file_name()
        .ok_or_else(|| "invalid source path".to_string())?
        .to_string_lossy()
        .to_string();

    let mut dest_dir = state.app_data_dir.clone();
    dest_dir.push("audio");
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    let dest = dest_dir.join(&filename);
    if !dest.exists() {
        fs::copy(&src, &dest).map_err(|e| format!("copy failed: {}", e))?;
        eprintln!("[file] copied {} → {}", source, dest.display());
    }
    Ok(dest.to_string_lossy().to_string())
}

/// Read a text file (e.g. .lrc lyrics) as a UTF-8 string.
#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("read failed: {}", e))
}

/// Open Finder at the file's location (macOS) or Explorer (Windows).
#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open Finder: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open Explorer: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("failed to open file manager: {}", e))?;
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .expect("failed to resolve app data dir");
            fs::create_dir_all(&app_data_dir)
                .expect("failed to create app data dir");

            let db_path = app_data_dir.join("data.db");
            eprintln!("[db] path: {}", db_path.display());

            let conn = Connection::open(&db_path)
                .expect("failed to open database");
            init_db(&conn)
                .expect("failed to initialize database");

            app.manage(DbState {
                conn: Mutex::new(conn),
                app_data_dir,
            });

            // macOS: use overlay titlebar style for native shadows + rounded corners
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_title_bar_style(TitleBarStyle::Overlay);
                }
            }

            // Build custom system menu — music player native
            #[cfg(desktop)]
            {
                let handle = app.handle().clone();
                build_menu(&handle, "进入全屏")?;

                // 监听窗口事件：全屏切换 + 关闭拦截
                if let Some(window) = app.get_webview_window("main") {
                    let handle = app.handle().clone();
                    window.on_window_event(move |event| {
                        match event {
                            tauri::WindowEvent::CloseRequested { api, .. } => {
                                // 拦截关闭：隐藏窗口而非退出
                                api.prevent_close();
                                if let Some(w) = handle.get_webview_window("main") {
                                    let _ = w.hide();
                                }
                            }
                            tauri::WindowEvent::Focused(focused) if *focused => {
                                if let Some(w) = handle.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            tauri::WindowEvent::Resized(_) => {
                                if let Some(w) = handle.get_webview_window("main") {
                                    let is_fs = w.is_fullscreen().unwrap_or(false);
                                    let _ = build_menu(&handle, if is_fs { "退出全屏" } else { "进入全屏" });
                                }
                            }
                            _ => {}
                        }
                    });
                }

                // 系统托盘
                let tray_menu = MenuBuilder::new(app.handle())
                    .item(&MenuItemBuilder::with_id("tray_play", "播放 / 暂停").build(app.handle())?)
                    .item(&MenuItemBuilder::with_id("tray_next", "下一首").build(app.handle())?)
                    .item(&MenuItemBuilder::with_id("tray_prev", "上一首").build(app.handle())?)
                    .separator()
                    .item(&MenuItemBuilder::with_id("tray_show", "显示窗口").build(app.handle())?)
                    .separator()
                    .item(&MenuItemBuilder::with_id("tray_quit", "退出 Hanono").build(app.handle())?)
                    .build()?;

                // 解码托盘图标
                let tray_icon_data = include_bytes!("../icons/tray-icon.png");
                let decoder = png::Decoder::new(std::io::Cursor::new(tray_icon_data));
                let mut reader = decoder.read_info().unwrap();
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).unwrap();
                let tray_image = tauri::image::Image::new_owned(buf, info.width, info.height);
                let _tray = TrayIconBuilder::new()
                    .icon(tray_image)
                    .icon_as_template(true)
                    .menu(&tray_menu)
                    .on_menu_event(|app, event| {
                        let id = event.id().as_ref();
                        match id {
                            "tray_play" | "tray_next" | "tray_prev" => {
                                let action = match id {
                                    "tray_play" => "play_pause",
                                    "tray_next" => "next",
                                    "tray_prev" => "prev",
                                    _ => return,
                                };
                                let _ = app.emit("menu-action", action);
                            }
                            "tray_show" => {
                                if let Some(w) = app.get_webview_window("main") {
                                    let _ = w.show();
                                    let _ = w.set_focus();
                                }
                            }
                            "tray_quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                            let _ = tray.app_handle().emit("menu-action", "play_pause");
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                // Playback controls
                "play_pause" | "next" | "prev" | "shuffle" | "repeat"
                | "vol_up" | "vol_down" => {
                    let _ = app.emit("menu-action", id);
                }
                // Playlist
                "import" | "favorite" | "clear" => {
                    let _ = app.emit("menu-action", id);
                }
                // About
                "about" => {
                    let _ = app.emit("menu-action", "about");
                }
                // Window controls
                "minimize" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.minimize();
                    }
                }
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.unminimize();
                        let _ = w.set_focus();
                    }
                }
                "zoom" => {
                    if let Some(w) = app.get_webview_window("main") {
                        if w.is_maximized().unwrap_or(false) {
                            let _ = w.unmaximize();
                        } else {
                            let _ = w.maximize();
                        }
                    }
                }
                "toggle_fullscreen" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let is_fullscreen = w.is_fullscreen().unwrap_or(false);
                        let _ = w.set_fullscreen(!is_fullscreen);
                        let _ = build_menu(app, if !is_fullscreen { "退出全屏" } else { "进入全屏" });
                    }
                }
                "toggle_ontop" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let is_top = w.is_always_on_top().unwrap_or(false);
                        let _ = w.set_always_on_top(!is_top);
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![save_playlist, load_playlist, save_favorites, load_favorites, copy_file_to_data, read_text_file, reveal_in_finder])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {});
}
