#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{OpenOptions, create_dir_all};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::{Duration, Instant};

use pixiv_platform_backend::api::{AppState, EnvPixivClientFactory, serve_listener};
use pixiv_platform_backend::pixiv::http::PixivHttpClient;
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
            refresh_pixiv_phpsessid,
            open_external_url
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
    #[cfg(target_os = "macos")]
    {
        select_download_directory_macos()
    }

    #[cfg(target_os = "windows")]
    {
        select_download_directory_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("folder picker is not supported on this platform yet".to_owned())
    }
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    const ALLOWED_RELEASES_URL: &str =
        "https://github.com/2921323707/self_pixiv_downloader/releases";
    if url != ALLOWED_RELEASES_URL {
        return Err("external URL is not allowed".to_owned());
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("/usr/bin/open")
            .arg(&url)
            .status()
            .map_err(|error| format!("browser could not be opened: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("browser open command failed with status {status}"))
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("rundll32.exe")
            .arg("url.dll,FileProtocolHandler")
            .arg(&url)
            .status()
            .map_err(|error| format!("browser could not be opened: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("browser open command failed with status {status}"))
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("opening external URLs is not supported on this platform yet".to_owned())
    }
}

#[cfg(target_os = "macos")]
fn select_download_directory_macos() -> Result<Option<String>, String> {
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

#[cfg(target_os = "windows")]
fn select_download_directory_windows() -> Result<Option<String>, String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = 'Choose a Pixiv Platform download folder'
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dialog.SelectedPath
}
"#;
    let output = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-STA")
        .arg("-Command")
        .arg(script)
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
        Err(format!("folder picker failed: {}", error.trim()))
    }
}

#[derive(Serialize)]
struct PixivSessionCookie {
    value: String,
    domain: Option<String>,
    path: Option<String>,
    http_only: Option<bool>,
    secure: Option<bool>,
    user_uid: String,
    user_name: Option<String>,
}

#[tauri::command]
async fn refresh_pixiv_phpsessid(
    app: tauri::AppHandle,
    logger: tauri::State<'_, DesktopLogger>,
) -> Result<PixivSessionCookie, String> {
    let logger = logger.inner().clone();
    logger.log("pixiv login refresh command invoked");
    let login_window = open_or_focus_pixiv_login_window(&app, &logger)?;

    tauri::async_runtime::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(180);
        let mut last_candidate_cookie: Option<String> = None;
        let mut next_validation_at = Instant::now();
        while Instant::now() < deadline {
            match login_window.cookies() {
                Ok(cookies) => {
                    if let Some(cookie) = cookies.into_iter().find(is_pixiv_phpsessid) {
                        let cookie_value = cookie.value().to_owned();
                        let should_validate = last_candidate_cookie.as_deref()
                            != Some(cookie_value.as_str())
                            || Instant::now() >= next_validation_at;
                        if !should_validate {
                            std::thread::sleep(Duration::from_millis(750));
                            continue;
                        }

                        logger.log(&format!(
                            "pixiv login window found candidate PHPSESSID cookie with length {}",
                            cookie_value.len()
                        ));

                        match validate_pixiv_login_cookie(&cookie_value) {
                            Ok(profile) => {
                                logger.log("pixiv login cookie verified");
                                let session_cookie = PixivSessionCookie {
                                    value: cookie_value,
                                    domain: cookie.domain().map(str::to_owned),
                                    path: cookie.path().map(str::to_owned),
                                    http_only: cookie.http_only(),
                                    secure: cookie.secure(),
                                    user_uid: profile.user_uid,
                                    user_name: profile.user_name,
                                };
                                if let Err(error) = login_window.close() {
                                    logger.log(&format!(
                                        "pixiv login window could not be closed after verified cookie refresh: {error}"
                                    ));
                                }
                                return Ok(session_cookie);
                            }
                            Err(error) => {
                                logger.log(&format!(
                                    "pixiv candidate PHPSESSID is not logged in yet: {error}"
                                ));
                                last_candidate_cookie = Some(cookie_value);
                                next_validation_at = Instant::now() + Duration::from_secs(3);
                            }
                        }
                    }
                }
                Err(error) => {
                    logger.log(&format!("pixiv login window cookie read failed: {error}"));
                }
            }
            std::thread::sleep(Duration::from_millis(750));
        }

        Err("Timed out waiting for a verified Pixiv login. Please finish signing in to Pixiv in the desktop window and try again.".to_owned())
    })
    .await
    .map_err(|error| format!("Pixiv login task failed: {error}"))?
}

fn validate_pixiv_login_cookie(
    cookie_value: &str,
) -> Result<pixiv_platform_backend::pixiv::PixivAccountProfile, String> {
    PixivHttpClient::new(cookie_value)
        .and_then(|client| client.fetch_current_user_profile())
        .map_err(|error| error.to_string())
}

fn open_or_focus_pixiv_login_window(
    app: &tauri::AppHandle,
    logger: &DesktopLogger,
) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window(PIXIV_LOGIN_WINDOW) {
        logger.log("pixiv login window already exists; focusing it");
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(window);
    }

    logger.log(&format!(
        "pixiv login window opening external URL {PIXIV_LOGIN_URL}"
    ));
    let url = Url::parse(PIXIV_LOGIN_URL).map_err(|error| error.to_string())?;
    WebviewWindowBuilder::new(app, PIXIV_LOGIN_WINDOW, WebviewUrl::External(url))
        .title("Pixiv Login")
        .inner_size(1100.0, 820.0)
        .min_inner_size(720.0, 560.0)
        .resizable(true)
        .build()
        .inspect(|_| logger.log("pixiv login window created"))
        .map_err(|error| {
            let message = format!("pixiv login window could not be created: {error}");
            logger.log(&message);
            message
        })
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
        .unwrap_or_else(|| default_download_root());
    let db_path = std::env::var_os("PIXIV_PLATFORM_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| download_root.join("pixiv_platform.sqlite3"));

    logger.log(&format!(
        "pixiv platform desktop storage using download root {} and db {}",
        download_root.display(),
        db_path.display()
    ));

    AppState::new(db_path, download_root, Arc::new(EnvPixivClientFactory))
}

fn default_download_root() -> PathBuf {
    home_dir()
        .map(PathBuf::from)
        .map(|home| home.join("Downloads/Pixiv Platform"))
        .unwrap_or_else(|| PathBuf::from("Pixiv Platform"))
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
    #[cfg(target_os = "macos")]
    {
        home_dir()
            .map(PathBuf::from)
            .map(|home| home.join("Library/Logs/Pixiv Platform/desktop.log"))
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(PathBuf::from))
            .map(|base| base.join("Pixiv Platform").join("desktop.log"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        home_dir()
            .map(PathBuf::from)
            .map(|home| home.join(".local/share/Pixiv Platform/desktop.log"))
    }
}

fn home_dir() -> Option<std::ffi::OsString> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
}
