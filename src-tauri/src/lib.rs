//! WaterForm on the desktop: one window onto a WaterForm server.
//!
//! The application itself is not in here and never will be. This is the window
//! around it: which server to talk to, what happens to a file the application
//! hands over, and how the shell replaces itself when there is a newer one. The
//! page the server serves gets **no** access to any of it — the capability in
//! `capabilities/shell.json` has no `remote` block, so a flaw in a page on the
//! server cannot reach the customer's machine through us.

mod i18n;
mod menu;
mod server;
mod settings;
mod update;

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::{AppHandle, Manager, State, Url, WebviewUrl, WebviewWindowBuilder, Wry};
use tauri_plugin_notification::NotificationExt;

use i18n::Lang;
use settings::Settings;

const MAIN: &str = "main";
const ABOUT: &str = "about";
/// Ten seconds after launch. Late enough that the application has the network to
/// itself while it loads, early enough that someone who opens the shell for five
/// minutes still hears about a new version.
const FIRST_CHECK: Duration = Duration::from_secs(10);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);

#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "windows";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const PLATFORM: &str = "linux";

pub struct Shell {
    /// The language everything the shell draws is in. Behind a lock because the
    /// customer can change it from the menu while the application is running.
    pub lang: Mutex<Lang>,
    /// What they chose, as opposed to what is being drawn: `None` means they are
    /// following the machine, which the menu has to be able to show.
    pub chosen_lang: Mutex<Option<Lang>>,
    pub config_dir: PathBuf,
    pub settings: Mutex<Settings>,
    /// Where the shell's own pages live. Read from the window once it exists
    /// rather than assembled from the platform, because the scheme differs
    /// between Windows and the rest and a guess would be a bug nobody sees
    /// until the day it matters.
    pub local_base: Mutex<Option<Url>>,
    /// Downloads in flight. macOS reports no path when one finishes, so the
    /// path we chose on the way in is the only one we will have to tell the
    /// customer where their file went.
    pub downloads: Mutex<VecDeque<PathBuf>>,
    pub update: update::UpdateState,
}

impl Shell {
    fn page(&self, page: &str, query: Option<&str>) -> Option<Url> {
        let mut url = self.local_base.lock().unwrap().clone()?;
        url.set_path(page);
        url.set_query(query);
        Some(url)
    }

    fn server_url(&self) -> Option<String> {
        self.settings.lock().unwrap().server_url.clone()
    }

    fn lang(&self) -> Lang {
        *self.lang.lock().unwrap()
    }
}

/* ── what the shell pages are told ─────────────────────────────────────────── */

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellState {
    lang: &'static str,
    platform: &'static str,
    version: String,
    default_server: String,
    server_url: Option<String>,
    /// What that server last said its own version was, so the About window can
    /// answer "which server am I on and what is running there" without a request.
    server_version: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Connected {
    url: String,
    version: String,
    mode: String,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Bootstrap {
    /// Nobody has been through the first-run screen on this machine.
    NeedsServer,
    /// The saved server answered and is ready; the page sends the window there.
    Ready(Connected),
    /// A server is saved and did not answer. The customer is told which one and
    /// why, on our page, instead of meeting the web view's own error screen.
    Unreachable { url: String, code: &'static str },
}

/* ── commands ──────────────────────────────────────────────────────────────── */

#[tauri::command]
fn shell_state(app: AppHandle, shell: State<'_, Shell>) -> ShellState {
    let (server_url, server_version) = {
        let settings = shell.settings.lock().unwrap();
        (settings.server_url.clone(), settings.last_version.clone())
    };
    ShellState {
        lang: shell.lang().code(),
        platform: PLATFORM,
        version: app.package_info().version.to_string(),
        default_server: server::DEFAULT_SERVER.to_string(),
        server_url,
        server_version,
    }
}

/// The first thing the window asks, every launch.
#[tauri::command]
async fn bootstrap(app: AppHandle) -> Bootstrap {
    let Some(url) = app.state::<Shell>().server_url() else {
        return Bootstrap::NeedsServer;
    };
    match server::probe(&url).await {
        Ok(health) => {
            remember(&app, &url, &health.version);
            Bootstrap::Ready(Connected {
                url,
                version: health.version,
                mode: health.mode,
            })
        }
        Err(error) => Bootstrap::Unreachable {
            url,
            code: probe_code(error),
        },
    }
}

/// Checks an address the customer typed and, if it holds a WaterForm server,
/// makes it the one this installation uses.
#[tauri::command]
async fn connect(app: AppHandle, url: String) -> Result<Connected, String> {
    let normalised = server::normalise(&url).map_err(|e| address_code(e).to_string())?;
    let health = server::probe(&normalised)
        .await
        .map_err(|e| probe_code(e).to_string())?;
    remember(&app, &normalised, &health.version);
    Ok(Connected {
        url: normalised,
        version: health.version,
        mode: health.mode,
    })
}

/// "Cancel" on the server screen: back to work if there is somewhere to go back
/// to, and otherwise there is no application to show, so the shell closes.
#[tauri::command]
fn cancel_connect(app: AppHandle) {
    match app.state::<Shell>().server_url() {
        Some(url) => {
            if let Ok(parsed) = Url::parse(&url) {
                if let Some(window) = app.get_webview_window(MAIN) {
                    let _ = window.navigate(parsed);
                }
            }
        }
        None => app.exit(0),
    }
}

#[tauri::command]
async fn check_updates(app: AppHandle) {
    update::check(app, true).await;
}

#[tauri::command]
async fn start_update(app: AppHandle) {
    update::install(app).await;
}

#[tauri::command]
fn later_update(app: AppHandle) {
    update::snooze(&app);
}

#[tauri::command]
fn restart_now(app: AppHandle) {
    app.restart();
}

/// The language the shell draws in: "tr", "en", or nothing to follow the machine.
#[tauri::command]
fn set_language(app: AppHandle, code: Option<String>) {
    apply_language(&app, code);
}

/// Opens one of the few addresses the About window carries, in the customer's
/// own browser. Only https and mailto: a shell page is ours, but a command that
/// opens whatever it is handed is a command worth not writing.
#[tauri::command]
fn open_link(url: String) -> Result<(), String> {
    let parsed = Url::parse(&url).map_err(|_| "not-an-address".to_string())?;
    match parsed.scheme() {
        "https" | "mailto" => {}
        _ => return Err("unsupported-scheme".to_string()),
    }
    tauri_plugin_opener::open_url(parsed.as_str(), None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn close_about(app: AppHandle) {
    if let Some(window) = app.get_webview_window(ABOUT) {
        let _ = window.close();
    }
}

/// Closes the update window without postponing anything: the states that use it
/// are the ones where there is nothing to postpone.
#[tauri::command]
fn close_update(app: AppHandle) {
    if let Some(window) = app.get_webview_window(update::WINDOW) {
        let _ = window.close();
    }
}

#[tauri::command]
fn update_status(shell: State<'_, Shell>) -> Option<update::Status> {
    shell.update.status.lock().unwrap().clone()
}

/* ── plumbing ──────────────────────────────────────────────────────────────── */

fn remember(app: &AppHandle, url: &str, version: &str) {
    let shell = app.state::<Shell>();
    let mut settings = shell.settings.lock().unwrap();
    settings.server_url = Some(url.to_string());
    settings.last_version = Some(version.to_string());
    if let Err(error) = settings.save(&shell.config_dir) {
        log::error!("could not save the server address: {error}");
    }
}

fn address_code(error: server::AddressError) -> &'static str {
    use server::AddressError::*;
    match error {
        Empty => "empty",
        Malformed => "malformed",
        UnsupportedScheme => "scheme",
        InsecurePublicHost => "insecure",
    }
}

fn probe_code(error: server::ProbeError) -> &'static str {
    use server::ProbeError::*;
    match error {
        Unreachable => "unreachable",
        Certificate => "certificate",
        NotWaterForm => "not-waterform",
        NotReady => "not-ready",
    }
}

/// A page that came out of this binary rather than off a server.
fn is_local(app: &AppHandle, url: &Url) -> bool {
    match url.scheme() {
        "http" | "https" => {}
        // tauri:, app:, blob:, data:, about: — the shell's own pages, and the
        // blob a finished export navigates to on its way to becoming a file.
        _ => return true,
    }

    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if host == "tauri.localhost" || host == "asset.localhost" {
        return true;
    }

    app.state::<Shell>()
        .local_base
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
        .as_deref()
        == Some(host.as_str())
}

/// Ours to show in the window, or somebody else's to open in a browser.
fn is_ours(app: &AppHandle, url: &Url) -> bool {
    if is_local(app, url) {
        return true;
    }
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    app.state::<Shell>()
        .server_url()
        .and_then(|s| Url::parse(&s).ok())
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
        .as_deref()
        == Some(host.as_str())
}

fn open_outside(url: &Url) {
    if let Err(error) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
        log::warn!("could not hand {url} to the browser: {error}");
    }
}

/// Where a file the application produced should land, and under what name.
fn download_target(app: &AppHandle, url: &Url, suggested: &Path) -> PathBuf {
    let dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .unwrap_or_else(|_| PathBuf::from("."));

    // The exports are blobs, so the name comes from the `download` attribute by
    // way of the web view; only when that is missing is there anything to work out.
    let name = suggested
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .or_else(|| {
            url.path_segments()
                .and_then(|mut s| s.next_back())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "waterform".to_string());

    unique_in(&dir, &sanitise(&name))
}

fn sanitise(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "/\\:*?\"<>|".contains(c) {
                '-'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "waterform".to_string()
    } else {
        trimmed
    }
}

/// Never writes over a file that is already there: the same tank exported twice
/// is two documents, not one.
fn unique_in(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for n in 2..1000 {
        let candidate = dir.join(format!("{stem} ({n}){ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(name)
}

/// A file that arrives with no sign of having arrived reads as an export that
/// failed, which is how support calls start.
fn announce_download(app: &AppHandle, path: &Path) {
    let strings = i18n::strings(app.state::<Shell>().lang());
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let result = app
        .notification()
        .builder()
        .title(strings.saved_title)
        .body(format!("{name}\n{}", strings.saved_body))
        .show();
    if let Err(error) = result {
        log::info!("saved {name}, could not show a notification: {error}");
    }
}

/// Puts a language choice into effect everywhere it shows.
///
/// The menu has to be rebuilt because its labels are baked in when it is made,
/// and any shell page that is open is reloaded so the window in front of the
/// customer changes with the setting rather than at the next launch. The
/// application's own page is left alone: it is the server's, and it follows the
/// account's language, not this one.
fn apply_language(app: &AppHandle, code: Option<String>) {
    let shell = app.state::<Shell>();
    {
        let mut settings = shell.settings.lock().unwrap();
        settings.lang = code.clone();
        if let Err(error) = settings.save(&shell.config_dir) {
            log::warn!("could not remember the language: {error}");
        }
    }

    let chosen = code.as_deref().and_then(Lang::from_code);
    let lang = i18n::resolve(code.as_deref());
    *shell.lang.lock().unwrap() = lang;
    *shell.chosen_lang.lock().unwrap() = chosen;

    match menu::build(app, lang, chosen) {
        Ok(menu) => {
            if let Err(error) = app.set_menu(menu) {
                log::warn!("could not redraw the menu: {error}");
            }
        }
        Err(error) => log::warn!("could not build the menu: {error}"),
    }

    for label in [ABOUT, update::WINDOW, MAIN] {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        let Ok(current) = window.url() else { continue };
        if label == MAIN && !is_local(app, &current) {
            continue;
        }
        let _ = window.navigate(current);
    }
}

fn open_about(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(ABOUT) {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    // The addresses on this window are opened in the customer's browser, not
    // here: a shell window that can wander onto a website stops being chrome.
    let nav = app.clone();
    let built = WebviewWindowBuilder::new(app, ABOUT, WebviewUrl::App("about.html".into()))
        .on_navigation(move |url| {
            if is_local(&nav, url) {
                true
            } else {
                open_outside(url);
                false
            }
        })
        .title("WaterForm")
        .inner_size(400.0, 580.0)
        .resizable(false)
        .maximizable(false)
        .center()
        .build();
    if let Err(error) = built {
        log::error!("could not open the about window: {error}");
    }
}

fn build_main(app: &AppHandle) -> tauri::Result<()> {
    let nav = app.clone();
    let popup = app.clone();
    let files = app.clone();

    let window = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("connect.html".into()))
        .title("WaterForm")
        .inner_size(1360.0, 900.0)
        .min_inner_size(1024.0, 640.0)
        .center()
        .on_navigation(move |url| {
            if is_ours(&nav, url) {
                true
            } else {
                open_outside(url);
                false
            }
        })
        .on_new_window(move |url, _features| {
            if is_ours(&popup, &url) {
                NewWindowResponse::Allow
            } else {
                open_outside(&url);
                NewWindowResponse::Deny
            }
        })
        .on_download(move |_webview, event| {
            match event {
                DownloadEvent::Requested { url, destination } => {
                    let target = download_target(&files, &url, destination);
                    log::info!("download -> {}", target.display());
                    files
                        .state::<Shell>()
                        .downloads
                        .lock()
                        .unwrap()
                        .push_back(target.clone());
                    *destination = target;
                }
                DownloadEvent::Finished { path, success, .. } => {
                    let remembered = files.state::<Shell>().downloads.lock().unwrap().pop_front();
                    let saved = path.filter(|p| !p.as_os_str().is_empty()).or(remembered);
                    match (success, saved) {
                        (true, Some(path)) => announce_download(&files, &path),
                        (true, None) => log::warn!("a download finished and did not say where"),
                        (false, _) => log::warn!("a download did not finish"),
                    }
                }
                _ => {}
            }
            true
        })
        .build()?;

    *app.state::<Shell>().local_base.lock().unwrap() = Some(window.url()?);
    Ok(())
}

fn open_connect_page(app: &AppHandle) {
    // The marker separates the two ways onto this page. At startup it checks the
    // saved server and hands over; from the menu it must not, because the person
    // who opened it came to change the address, not to be sent back.
    let Some(page) = app.state::<Shell>().page("connect.html", Some("change")) else {
        return;
    };
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.navigate(page);
    }
}

fn reload_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN) {
        if let Ok(current) = window.url() {
            let _ = window.navigate(current);
        }
    }
}

fn open_downloads_folder(app: &AppHandle) {
    if let Ok(dir) = app.path().download_dir() {
        if let Err(error) =
            tauri_plugin_opener::open_path(dir.to_string_lossy().to_string(), None::<&str>)
        {
            log::warn!("could not open the downloads folder: {error}");
        }
    }
}

/// Looks for a new shell shortly after launch and every six hours after that.
/// Both are quiet when there is nothing to say or nothing to reach.
fn watch_for_updates(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK).await;
        loop {
            update::check(app.clone(), false).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .on_menu_event(|app, event| match event.id().as_ref() {
            menu::ID_SERVER => open_connect_page(app),
            menu::ID_ABOUT => open_about(app),
            menu::ID_LANG_SYSTEM => apply_language(app, None),
            menu::ID_LANG_TR => apply_language(app, Some("tr".into())),
            menu::ID_LANG_EN => apply_language(app, Some("en".into())),
            menu::ID_RELOAD => reload_main(app),
            menu::ID_DOWNLOADS => open_downloads_folder(app),
            menu::ID_UPDATES => {
                let app = app.clone();
                tauri::async_runtime::spawn(update::check(app, true));
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            shell_state,
            bootstrap,
            connect,
            cancel_connect,
            check_updates,
            start_update,
            later_update,
            restart_now,
            update_status,
            close_update,
            set_language,
            open_link,
            close_about,
        ])
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let stored = Settings::load(&config_dir);
            let chosen = stored.lang.as_deref().and_then(Lang::from_code);
            let lang = i18n::resolve(stored.lang.as_deref());
            log::info!(
                "starting, server = {:?}, language = {}",
                stored.server_url,
                lang.code()
            );

            app.manage(Shell {
                lang: Mutex::new(lang),
                chosen_lang: Mutex::new(chosen),
                config_dir,
                settings: Mutex::new(stored),
                local_base: Mutex::new(None),
                downloads: Mutex::new(VecDeque::new()),
                update: update::UpdateState::default(),
            });

            let handle = app.handle().clone();
            app.set_menu(menu::build(&handle, lang, chosen)?)?;
            build_main(&handle)?;
            watch_for_updates(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the shell could not start");
}

/// Kept honest by the compiler: `NewWindowResponse` is generic over the runtime,
/// and the shell only ever runs on the default one.
const _: fn() -> NewWindowResponse<Wry> = || NewWindowResponse::Deny;
