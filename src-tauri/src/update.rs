//! Keeping the shell current.
//!
//! The window the customer works in is never touched by this: the offer, the
//! progress and the restart all happen in a small window of the shell's own, so
//! an update can never interrupt a drawing half way through, and nothing is
//! injected into the page the server serves.

use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::i18n;
use crate::Shell;

/// A day. Long enough that "later" means later, short enough that a customer who
/// keeps the application open all week still gets asked again.
const SNOOZE_SECONDS: i64 = 24 * 60 * 60;

pub const EVENT: &str = "wf://update";
pub const WINDOW: &str = "update";

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Status {
    Checking,
    UpToDate,
    Available {
        version: String,
        current: String,
        notes: Option<String>,
    },
    Downloading {
        received: u64,
        total: Option<u64>,
    },
    Ready,
    Failed {
        message: String,
    },
}

#[derive(Default)]
pub struct UpdateState {
    pub status: Mutex<Option<Status>>,
    /// The update we have been offered and not yet installed. Held rather than
    /// re-fetched so that pressing "Update" cannot pick up a different release
    /// from the one the customer was shown.
    pub pending: Mutex<Option<Update>>,
}

fn publish(app: &AppHandle, status: Status) {
    if let Some(shell) = app.try_state::<Shell>() {
        *shell.update.status.lock().unwrap() = Some(status.clone());
    }
    if let Err(error) = app.emit_to(WINDOW, EVENT, status) {
        log::debug!("no update window to tell: {error}");
    }
}

fn open_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::App("update.html".into()))
        .title("WaterForm")
        .inner_size(460.0, 260.0)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .center()
        .always_on_top(true)
        .build();
    if let Err(error) = built {
        log::error!("could not open the update window: {error}");
    }
}

/// Where to ask. Normally the two endpoints in `tauri.conf.json`; an install
/// with no way out to the internet can be pointed at a mirror on its own network
/// with `WF_UPDATE_ENDPOINT`. That is not a way in: the package still has to
/// carry a signature made with our key or the updater refuses to install it.
fn build_updater(app: &AppHandle) -> tauri_plugin_updater::Result<tauri_plugin_updater::Updater> {
    let mut builder = app.updater_builder();
    if let Ok(endpoint) = std::env::var("WF_UPDATE_ENDPOINT") {
        match endpoint.parse() {
            Ok(url) => {
                log::info!("update endpoint overridden: {endpoint}");
                builder = builder.endpoints(vec![url])?;
            }
            Err(error) => log::warn!("WF_UPDATE_ENDPOINT is not an address: {error}"),
        }
    }
    builder.build()
}

/// Asks the endpoints whether there is anything newer.
///
/// `manual` separates the two reasons we ask. A scheduled check that finds
/// nothing, or cannot reach anything, says nothing at all: a customer on a
/// closed factory network would otherwise be told off every six hours for a
/// network we knew they did not have. A check the customer asked for always
/// answers, because silence would read as a broken button.
pub async fn check(app: AppHandle, manual: bool) {
    let lang = app.state::<Shell>().lang();
    let strings = i18n::strings(lang);

    if !manual {
        let shell = app.state::<Shell>();
        let snoozing = shell.settings.lock().unwrap().snoozing(now());
        if snoozing {
            log::info!("update check skipped: the customer asked for it later");
            return;
        }
    }

    if manual {
        open_window(&app);
        publish(&app, Status::Checking);
    }

    let updater = match build_updater(&app) {
        Ok(updater) => updater,
        Err(error) => return report_failure(&app, manual, strings, error.to_string()),
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let status = Status::Available {
                version: update.version.clone(),
                current: update.current_version.clone(),
                notes: update.body.clone(),
            };
            *app.state::<Shell>().update.pending.lock().unwrap() = Some(update);
            open_window(&app);
            publish(&app, status);
        }
        Ok(None) => {
            if manual {
                publish(&app, Status::UpToDate);
            } else {
                log::info!("already on the newest version");
            }
        }
        Err(error) => report_failure(&app, manual, strings, error.to_string()),
    }
}

fn report_failure(app: &AppHandle, manual: bool, strings: &i18n::Strings, message: String) {
    if manual {
        publish(
            app,
            Status::Failed {
                message: message.clone(),
            },
        );
    }
    log::warn!("{}: {message}", strings.update_failed_title);
}

/// Downloads and installs the update the customer has just accepted.
pub async fn install(app: AppHandle) {
    let update = app.state::<Shell>().update.pending.lock().unwrap().take();
    let Some(update) = update else {
        publish(
            &app,
            Status::Failed {
                message: "no update is waiting".into(),
            },
        );
        return;
    };

    let mut received: u64 = 0;
    let progress_app = app.clone();
    let finished_app = app.clone();

    let outcome = update
        .download_and_install(
            move |chunk, total| {
                received += chunk as u64;
                publish(&progress_app, Status::Downloading { received, total });
            },
            move || {
                publish(&finished_app, Status::Ready);
            },
        )
        .await;

    match outcome {
        Ok(()) => publish(&app, Status::Ready),
        Err(error) => {
            log::error!("update failed: {error}");
            publish(
                &app,
                Status::Failed {
                    message: error.to_string(),
                },
            );
        }
    }
}

/// "Later": the window closes and nothing is said for a day.
pub fn snooze(app: &AppHandle) {
    let shell = app.state::<Shell>();
    {
        let mut settings = shell.settings.lock().unwrap();
        settings.update_snoozed_until = Some(now() + SNOOZE_SECONDS);
        if let Err(error) = settings.save(&shell.config_dir) {
            log::warn!("could not remember the postponed update: {error}");
        }
    }
    *shell.update.pending.lock().unwrap() = None;
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.close();
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}
