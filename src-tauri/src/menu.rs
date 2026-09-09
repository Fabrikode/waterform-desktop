//! The window's own menu.
//!
//! The server address lives here rather than on a gear in the application,
//! because "which server am I talking to" is the shell's business and the
//! application's own Ayarlar belongs to the customer's company. Putting both in
//! one place would mean a setting that exists in the browser too but does
//! nothing there.

use tauri::menu::{
    AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Runtime};

use crate::i18n::{self, Lang};

pub const ID_SERVER: &str = "wf.server";
pub const ID_UPDATES: &str = "wf.updates";
pub const ID_RELOAD: &str = "wf.reload";
pub const ID_DOWNLOADS: &str = "wf.downloads";

pub fn build<R: Runtime>(app: &AppHandle<R>, lang: Lang) -> tauri::Result<Menu<R>> {
    let s = i18n::strings(lang);

    let server = MenuItemBuilder::with_id(ID_SERVER, s.server)
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let updates = MenuItemBuilder::with_id(ID_UPDATES, s.check_updates).build(app)?;
    let reload = MenuItemBuilder::with_id(ID_RELOAD, s.reload)
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    let downloads = MenuItemBuilder::with_id(ID_DOWNLOADS, s.downloads_folder).build(app)?;

    let about_metadata = AboutMetadata {
        name: Some("WaterForm".into()),
        version: Some(app.package_info().version.to_string()),
        copyright: Some("© Fabrikode".into()),
        website: Some("https://waterform.fabrikode.com".into()),
        website_label: Some("waterform.fabrikode.com".into()),
        ..Default::default()
    };

    let edit = SubmenuBuilder::new(app, s.menu_edit)
        .item(&PredefinedMenuItem::undo(app, Some(s.undo))?)
        .item(&PredefinedMenuItem::redo(app, Some(s.redo))?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, Some(s.cut))?)
        .item(&PredefinedMenuItem::copy(app, Some(s.copy))?)
        .item(&PredefinedMenuItem::paste(app, Some(s.paste))?)
        .item(&PredefinedMenuItem::select_all(app, Some(s.select_all))?)
        .build()?;

    #[cfg(target_os = "macos")]
    {
        // On macOS the first submenu is the application menu, and the platform
        // expects About, Services, Hide and Quit to be in it. Anything else here
        // reads as a foreign application.
        let app_menu = SubmenuBuilder::new(app, s.menu_app)
            .item(&PredefinedMenuItem::about(
                app,
                Some(s.about),
                Some(about_metadata),
            )?)
            .separator()
            .item(&server)
            .item(&updates)
            .separator()
            .item(&PredefinedMenuItem::services(app, None)?)
            .separator()
            .item(&PredefinedMenuItem::hide(app, Some(s.hide))?)
            .item(&PredefinedMenuItem::hide_others(app, Some(s.hide_others))?)
            .separator()
            .item(&PredefinedMenuItem::quit(app, Some(s.quit))?)
            .build()?;

        let file = SubmenuBuilder::new(app, s.menu_file)
            .item(&downloads)
            .separator()
            .item(&PredefinedMenuItem::close_window(app, Some(s.close))?)
            .build()?;

        let view = SubmenuBuilder::new(app, s.menu_view)
            .item(&reload)
            .separator()
            .item(&PredefinedMenuItem::fullscreen(app, Some(s.fullscreen))?)
            .item(&PredefinedMenuItem::minimize(app, Some(s.minimize))?)
            .build()?;

        MenuBuilder::new(app)
            .item(&app_menu)
            .item(&file)
            .item(&edit)
            .item(&view)
            .build()
    }

    #[cfg(not(target_os = "macos"))]
    {
        let file = SubmenuBuilder::new(app, s.menu_file)
            .item(&server)
            .item(&downloads)
            .separator()
            .item(&PredefinedMenuItem::quit(app, Some(s.quit))?)
            .build()?;

        let view = SubmenuBuilder::new(app, s.menu_view)
            .item(&reload)
            .item(&PredefinedMenuItem::minimize(app, Some(s.minimize))?)
            .build()?;

        let help = SubmenuBuilder::new(app, s.menu_help)
            .item(&updates)
            .separator()
            .item(&PredefinedMenuItem::about(
                app,
                Some(s.about),
                Some(about_metadata),
            )?)
            .build()?;

        MenuBuilder::new(app)
            .item(&file)
            .item(&edit)
            .item(&view)
            .item(&help)
            .build()
    }
}
