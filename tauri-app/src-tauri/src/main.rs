#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self, OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::{Duration, Instant};

use pixiv_platform_backend::api::{AppState, EnvPixivClientFactory, serve_listener};
use pixiv_platform_backend::db;
use serde::Serialize;
use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::webview::Cookie;
use tauri::{Manager, Url, WebviewUrl, WebviewWindowBuilder};

const MENU_RELOAD: &str = "pixiv-platform-reload";
const PIXIV_LOGIN_WINDOW: &str = "pixiv-login";
const PIXIV_LOGIN_URL: &str = "https://www.pixiv.net/";
#[cfg(debug_assertions)]
const MENU_TOGGLE_DEVTOOLS: &str = "pixiv-platform-toggle-devtools";

fn main() {
    let logger = DesktopLogger::new();
    logger.log("pixiv platform desktop app starting");

    tauri::Builder::default()
        .menu(create_app_menu)
        .on_menu_event(handle_menu_event)
        .manage(logger.clone())
        .invoke_handler(tauri::generate_handler![
            select_download_directory,
            refresh_pixiv_phpsessid
        ])
        .setup(move |app| {
            let logger = logger.clone();
            match start_desktop(app, &logger) {
                Ok(()) => {}
                Err(error) => {
                    let message = error.to_string();
                    logger.log(&format!("startup failed: {message}"));
                    create_startup_error_window(app, &message, logger.path())?;
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Pixiv Platform desktop app");
}

fn create_app_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let package_info = app.package_info();
    let about_metadata = AboutMetadata {
        name: Some("Pixiv Platform".to_owned()),
        version: Some(package_info.version.to_string()),
        ..Default::default()
    };

    let app_menu = SubmenuBuilder::new(app, "Pixiv Platform")
        .about(Some(about_metadata))
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .separator()
        .quit()
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File").close_window().build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&MenuItemBuilder::with_id(MENU_RELOAD, "Reload").build(app)?);
    #[cfg(debug_assertions)]
    let view_menu = view_menu
        .item(&MenuItemBuilder::with_id(MENU_TOGGLE_DEVTOOLS, "Developer Tools").build(app)?);
    let view_menu = view_menu.separator().fullscreen().build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .build()
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    if event.id() == MENU_RELOAD {
        if let Some(window) = app.get_webview_window("main") {
            if let Err(error) = window.reload() {
                eprintln!("pixiv platform desktop menu reload failed: {error}");
            }
        }
    }

    #[cfg(debug_assertions)]
    if event.id() == MENU_TOGGLE_DEVTOOLS {
        if let Some(window) = app.get_webview_window("main") {
            if window.is_devtools_open() {
                window.close_devtools();
            } else {
                window.open_devtools();
            }
        }
    }
}

#[tauri::command]
fn select_download_directory() -> Result<Option<String>, String> {
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(
            "POSIX path of (choose folder with prompt \"Choose a Pixiv Platform download folder\")",
        )
        .output()
        .map_err(|error| format!("folder picker could not be opened: {error}"))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if path.is_empty() {
            Ok(None)
        } else {
            Ok(Some(path))
        }
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("User canceled") || error.contains("-128") {
            Ok(None)
        } else {
            Err(format!("folder picker failed: {}", error.trim()))
        }
    }
}

#[derive(Serialize)]
struct PixivSessionCookie {
    value: String,
    domain: Option<String>,
    path: Option<String>,
    http_only: Option<bool>,
    secure: Option<bool>,
}

#[tauri::command]
async fn refresh_pixiv_phpsessid(
    app: tauri::AppHandle,
    logger: tauri::State<'_, DesktopLogger>,
) -> Result<PixivSessionCookie, String> {
    let login_window = open_or_focus_pixiv_login_window(&app)?;
    let logger = logger.inner().clone();

    tauri::async_runtime::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(180);
        while Instant::now() < deadline {
            match login_window.cookies() {
                Ok(cookies) => {
                    if let Some(cookie) = cookies.into_iter().find(is_pixiv_phpsessid) {
                        logger.log(&format!(
                            "pixiv login window found PHPSESSID cookie with length {}",
                            cookie.value().len()
                        ));
                        let session_cookie = PixivSessionCookie {
                            value: cookie.value().to_owned(),
                            domain: cookie.domain().map(str::to_owned),
                            path: cookie.path().map(str::to_owned),
                            http_only: cookie.http_only(),
                            secure: cookie.secure(),
                        };
                        if let Err(error) = login_window.close() {
                            logger.log(&format!(
                                "pixiv login window could not be closed after cookie refresh: {error}"
                            ));
                        }
                        return Ok(session_cookie);
                    }
                }
                Err(error) => {
                    logger.log(&format!("pixiv login window cookie read failed: {error}"));
                }
            }
            std::thread::sleep(Duration::from_millis(750));
        }

        Err("Timed out waiting for Pixiv PHPSESSID. Please finish Pixiv login in the desktop window and try again.".to_owned())
    })
    .await
    .map_err(|error| format!("Pixiv login task failed: {error}"))?
}

fn open_or_focus_pixiv_login_window(
    app: &tauri::AppHandle,
) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(PIXIV_LOGIN_WINDOW) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(window);
    }

    let url = Url::parse(PIXIV_LOGIN_URL).map_err(|error| error.to_string())?;
    WebviewWindowBuilder::new(app, PIXIV_LOGIN_WINDOW, WebviewUrl::External(url))
        .title("Pixiv Login")
        .inner_size(1100.0, 820.0)
        .min_inner_size(720.0, 560.0)
        .resizable(true)
        .build()
        .map_err(|error| error.to_string())
}

fn is_pixiv_phpsessid(cookie: &Cookie<'_>) -> bool {
    cookie.name() == "PHPSESSID" && cookie.domain().is_some_and(is_pixiv_cookie_domain)
}

fn is_pixiv_cookie_domain(domain: &str) -> bool {
    let domain = domain.trim_start_matches('.').to_ascii_lowercase();
    domain == "pixiv.net" || domain.ends_with(".pixiv.net")
}

fn start_desktop(
    app: &tauri::App,
    logger: &DesktopLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend_addr = start_backend(logger)?;
    wait_for_backend_health(backend_addr, logger)?;
    create_main_window(app, backend_addr, logger)?;
    Ok(())
}

fn start_backend(logger: &DesktopLogger) -> Result<SocketAddr, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    let logger = logger.clone();

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                logger.log(&format!("desktop backend runtime failed to start: {error}"));
                return;
            }
        };

        runtime.block_on(async {
            let state = desktop_app_state(&logger);
            let listener = match tokio::net::TcpListener::from_std(listener) {
                Ok(listener) => listener,
                Err(error) => {
                    logger.log(&format!(
                        "desktop backend listener was not accepted by Tokio: {error}"
                    ));
                    return;
                }
            };

            logger.log(&format!(
                "pixiv platform desktop backend starting on http://{addr}"
            ));
            if let Err(error) = serve_listener(state, listener).await {
                logger.log(&format!("pixiv platform desktop backend stopped: {error}"));
            }
        });
    });

    Ok(addr)
}

fn desktop_app_state(logger: &DesktopLogger) -> AppState {
    let download_root = std::env::var_os("PIXIV_DOWNLOAD_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| desktop_download_root().unwrap_or_else(|| PathBuf::from("output")));
    let db_path = std::env::var_os("PIXIV_PLATFORM_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| download_root.join("pixiv_platform.sqlite3"));

    if std::env::var_os("PIXIV_PLATFORM_DB_PATH").is_none() {
        migrate_legacy_desktop_data(&db_path, &download_root, logger);
    }

    logger.log(&format!(
        "pixiv platform desktop storage using download root {} and db {}",
        download_root.display(),
        db_path.display()
    ));

    AppState::new(db_path, download_root, Arc::new(EnvPixivClientFactory))
}

fn migrate_legacy_desktop_data(db_path: &PathBuf, download_root: &PathBuf, logger: &DesktopLogger) {
    let Some(legacy_root) = legacy_project_output_dir() else {
        return;
    };
    let legacy_db_path = legacy_root.join("pixiv_platform.sqlite3");
    if !legacy_db_path.exists() || legacy_db_path == *db_path {
        return;
    }

    let should_restore = if db_path.exists() {
        desktop_db_looks_empty(db_path, logger) && legacy_db_has_user_data(&legacy_db_path, logger)
    } else {
        true
    };
    if !should_restore {
        return;
    }

    if let Some(parent) = db_path.parent() {
        if let Err(error) = create_dir_all(parent) {
            logger.log(&format!(
                "desktop legacy data migration skipped because target directory could not be created: {error}"
            ));
            return;
        }
    }

    if db_path.exists() {
        let backup_path = db_path.with_extension(format!(
            "sqlite3.empty-before-legacy-migration-{}",
            unix_timestamp()
        ));
        match fs::copy(db_path, &backup_path) {
            Ok(_) => logger.log(&format!(
                "desktop legacy data migration backed up existing empty db to {}",
                backup_path.display()
            )),
            Err(error) => {
                logger.log(&format!(
                    "desktop legacy data migration skipped because existing db backup failed: {error}"
                ));
                return;
            }
        }
    }

    if let Err(error) = copy_legacy_download_files(&legacy_root, download_root, logger) {
        logger.log(&format!(
            "desktop legacy data migration could not copy all downloaded files: {error}"
        ));
    }

    match fs::copy(&legacy_db_path, db_path) {
        Ok(_) => {
            logger.log(&format!(
                "desktop legacy data migration copied db from {} to {}",
                legacy_db_path.display(),
                db_path.display()
            ));
            rewrite_migrated_image_paths(db_path, &legacy_root, download_root, logger);
        }
        Err(error) => logger.log(&format!(
            "desktop legacy data migration failed to copy db from {}: {error}",
            legacy_db_path.display()
        )),
    }
}

fn desktop_db_looks_empty(db_path: &PathBuf, logger: &DesktopLogger) -> bool {
    match db_summary(db_path) {
        Ok(summary) => summary.image_count == 0 && !summary.has_pixiv_cookie,
        Err(error) => {
            logger.log(&format!(
                "desktop legacy data migration could not inspect target db: {error}"
            ));
            false
        }
    }
}

fn legacy_db_has_user_data(db_path: &PathBuf, logger: &DesktopLogger) -> bool {
    match db_summary(db_path) {
        Ok(summary) => summary.image_count > 0 || summary.has_pixiv_cookie,
        Err(error) => {
            logger.log(&format!(
                "desktop legacy data migration could not inspect legacy db: {error}"
            ));
            false
        }
    }
}

struct DbSummary {
    image_count: i64,
    has_pixiv_cookie: bool,
}

fn db_summary(db_path: &PathBuf) -> Result<DbSummary, String> {
    let conn = db::open(db_path).map_err(|error| error.to_string())?;
    let image_count = conn
        .query_row("SELECT COUNT(*) FROM images", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|error| error.to_string())?;
    let pixiv_cookie_count = conn
        .query_row(
            "SELECT COUNT(*) FROM settings WHERE key = 'pixiv_cookie'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(DbSummary {
        image_count,
        has_pixiv_cookie: pixiv_cookie_count > 0,
    })
}

fn copy_legacy_download_files(
    legacy_root: &PathBuf,
    download_root: &PathBuf,
    logger: &DesktopLogger,
) -> Result<(), std::io::Error> {
    if !legacy_root.exists() {
        return Ok(());
    }
    create_dir_all(download_root)?;
    copy_directory_contents(legacy_root, download_root, logger)
}

fn copy_directory_contents(
    source: &PathBuf,
    destination: &PathBuf,
    logger: &DesktopLogger,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        if file_name == "pixiv_platform.sqlite3" {
            continue;
        }

        let destination_path = destination.join(file_name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            create_dir_all(&destination_path)?;
            copy_directory_contents(&source_path, &destination_path, logger)?;
        } else if file_type.is_file() && !destination_path.exists() {
            fs::copy(&source_path, &destination_path)?;
        } else if !file_type.is_file() {
            logger.log(&format!(
                "desktop legacy data migration skipped non-file path {}",
                source_path.display()
            ));
        }
    }
    Ok(())
}

fn rewrite_migrated_image_paths(
    db_path: &PathBuf,
    legacy_root: &PathBuf,
    download_root: &PathBuf,
    logger: &DesktopLogger,
) {
    let legacy_prefix = legacy_root.to_string_lossy().to_string();
    let download_prefix = download_root.to_string_lossy().to_string();
    match db::open(db_path).and_then(|conn| {
        conn.execute(
            "UPDATE images
             SET local_path = replace(local_path, ?1, ?2)
             WHERE local_path LIKE ?3",
            (
                legacy_prefix.as_str(),
                download_prefix.as_str(),
                format!("{legacy_prefix}%"),
            ),
        )?;
        Ok(())
    }) {
        Ok(()) => logger.log("desktop legacy data migration rewrote migrated image paths"),
        Err(error) => logger.log(&format!(
            "desktop legacy data migration could not rewrite image paths: {error}"
        )),
    }
}

fn legacy_project_output_dir() -> Option<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(|root| root.join("output"))
}

fn desktop_download_root() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Downloads/Pixiv Platform"))
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn wait_for_backend_health(addr: SocketAddr, logger: &DesktopLogger) -> Result<(), std::io::Error> {
    let health_path = "/api/health";
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut last_error = String::from("health check was not attempted");

    while Instant::now() < deadline {
        match probe_backend_health(addr, health_path) {
            Ok(()) => {
                logger.log(&format!(
                    "pixiv platform desktop backend is healthy at http://{addr}{health_path}"
                ));
                return Ok(());
            }
            Err(error) => {
                last_error = error.to_string();
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    let message = format!(
        "pixiv platform desktop backend health check failed for http://{addr}{health_path}: timed out after 8s; last error: {last_error}"
    );
    logger.log(&message);
    Err(std::io::Error::new(std::io::ErrorKind::TimedOut, message))
}

fn probe_backend_health(addr: SocketAddr, path: &str) -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(250))?;
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    stream.set_write_timeout(Some(Duration::from_millis(250)))?;

    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;

    let mut response = [0_u8; 512];
    let bytes_read = stream.read(&mut response)?;
    let response = String::from_utf8_lossy(&response[..bytes_read]);

    if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "unexpected health response: {}",
            response.lines().next().unwrap_or("<empty response>")
        )))
    }
}

fn create_main_window(
    app: &tauri::App,
    backend_addr: SocketAddr,
    logger: &DesktopLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend_url = format!("http://{backend_addr}");
    let init_script = format!(r#"window.__PIXIV_PLATFORM_BACKEND_URL__ = "{backend_url}";"#);

    WebviewWindowBuilder::new(app, "main", main_window_url())
        .title("Pixiv Platform")
        .inner_size(1280.0, 820.0)
        .min_inner_size(1024.0, 680.0)
        .resizable(true)
        .initialization_script(init_script)
        .build()?;

    logger.log("pixiv platform desktop main window created");
    Ok(())
}

fn create_startup_error_window(
    app: &tauri::App,
    message: &str,
    log_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = log_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "unavailable".to_owned());
    let init_script = format!(
        "window.__PIXIV_PLATFORM_STARTUP_ERROR__ = {{ message: '{}', logPath: '{}' }};",
        escape_js_string(message),
        escape_js_string(&log_path)
    );

    WebviewWindowBuilder::new(app, "startup-error", startup_error_window_url())
        .title("Pixiv Platform Startup Error")
        .inner_size(720.0, 420.0)
        .min_inner_size(560.0, 360.0)
        .resizable(true)
        .initialization_script(init_script)
        .build()?;

    Ok(())
}

fn main_window_url() -> WebviewUrl {
    if cfg!(debug_assertions) {
        WebviewUrl::External(
            Url::parse("http://127.0.0.1:3001").expect("desktop dev URL should be valid"),
        )
    } else {
        WebviewUrl::App("index.html".into())
    }
}

fn startup_error_window_url() -> WebviewUrl {
    if cfg!(debug_assertions) {
        WebviewUrl::External(
            Url::parse("http://127.0.0.1:3001/startup-error.html")
                .expect("desktop startup error URL should be valid"),
        )
    } else {
        WebviewUrl::App("startup-error.html".into())
    }
}

fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

#[derive(Clone)]
struct DesktopLogger {
    path: Arc<Option<PathBuf>>,
}

impl DesktopLogger {
    fn new() -> Self {
        Self {
            path: Arc::new(desktop_log_path()),
        }
    }

    fn path(&self) -> Option<PathBuf> {
        self.path.as_ref().clone()
    }

    fn log(&self, message: &str) {
        eprintln!("{message}");

        let Some(path) = self.path.as_ref() else {
            return;
        };

        if let Some(parent) = path.parent() {
            if let Err(error) = create_dir_all(parent) {
                eprintln!("pixiv platform desktop log directory could not be created: {error}");
                return;
            }
        }

        match OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut file) => {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|duration| duration.as_secs())
                    .unwrap_or_default();
                if let Err(error) = writeln!(file, "{timestamp} {message}") {
                    eprintln!("pixiv platform desktop log file could not be written: {error}");
                }
            }
            Err(error) => {
                eprintln!("pixiv platform desktop log file could not be opened: {error}");
            }
        }
    }
}

fn desktop_log_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Logs/Pixiv Platform/desktop.log"))
}
