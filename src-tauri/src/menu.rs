//! The window's own menu.
//!
//! The server address lives here rather than on a gear in the application,
//! because "which server am I talking to" is the shell's business and the
//! application's own Ayarlar belongs to the customer's company. Putting both in
//! one place would mean a setting that exists in the browser too but does
//! nothing there.

use tauri::menu::{
    CheckMenuItemBuilder, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Runtime};

use crate::i18n::{self, Lang};

pub const ID_SERVER: &str = "wf.server";
pub const ID_ABOUT: &str = "wf.about";
pub const ID_LANG_SYSTEM: &str = "wf.lang.system";
pub const ID_LANG_TR: &str = "wf.lang.tr";
pub const ID_LANG_EN: &str = "wf.lang.en";
pub const ID_UPDATES: &str = "wf.updates";
pub const ID_RELOAD: &str = "wf.reload";
pub const ID_DOWNLOADS: &str = "wf.downloads";

/// `chosen` is what the customer picked, and `None` means they have not: the
/// menu shows which of the three is in force rather than only the language being
/// drawn, so "System language" stays distinguishable from "Turkish" on a Turkish
/// machine.
pub fn build<R: Runtime>(
    app: &AppHandle<R>,
    lang: Lang,
    chosen: Option<Lang>,
) -> tauri::Result<Menu<R>> {
    let s = i18n::strings(lang);

    let server = MenuItemBuilder::with_id(ID_SERVER, s.server)
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let updates = MenuItemBuilder::with_id(ID_UPDATES, s.check_updates).build(app)?;
    let reload = MenuItemBuilder::with_id(ID_RELOAD, s.reload)
        .accelerator("CmdOrCtrl+R")
        .build(app)?;
    let downloads = MenuItemBuilder::with_id(ID_DOWNLOADS, s.downloads_folder).build(app)?;

    // Our own window rather than the platform's About panel: the panel cannot
    // carry a link anyone can click, and the two things a customer needs from it
    // are the address of the server they are on and a way to reach us.
    let about = MenuItemBuilder::with_id(ID_ABOUT, s.about).build(app)?;

    let language = SubmenuBuilder::new(app, s.menu_language)
        .item(
            &CheckMenuItemBuilder::with_id(ID_LANG_SYSTEM, s.lang_system)
                .checked(chosen.is_none())
                .build(app)?,
        )
        .item(
            &CheckMenuItemBuilder::with_id(ID_LANG_TR, "Türkçe")
                .checked(chosen == Some(Lang::Tr))
                .build(app)?,
        )
        .item(
            &CheckMenuItemBuilder::with_id(ID_LANG_EN, "English")
                .checked(chosen == Some(Lang::En))
                .build(app)?,
        )
        .build()?;

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
            .item(&about)
            .separator()
            .item(&server)
            .item(&language)
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
            .item(&language)
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
            .item(&about)
            .build()?;

        MenuBuilder::new(app)
            .item(&file)
            .item(&edit)
            .item(&view)
            .item(&help)
            .build()
    }
}
