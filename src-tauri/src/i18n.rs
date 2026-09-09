//! Turkish and English for the parts of the shell the operating system draws.
//!
//! The application inside the window speaks whatever language the account is
//! set to; the menu bar and the shell's own dialogs belong to the machine, so
//! they follow the machine. Two languages, chosen once at startup, no runtime
//! switch: a menu that changes language while you are using it is worse than
//! one that picked wrong.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Tr,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Tr => "tr",
            Lang::En => "en",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "tr" => Some(Lang::Tr),
            "en" => Some(Lang::En),
            _ => None,
        }
    }
}

/// The language to draw in: what the customer chose, or failing that what the
/// machine is set to.
///
/// The choice exists because the machine is a poor guess for this product. A
/// Turkish engineer is often given an English Windows install by whoever set the
/// office up, and a company that sells tanks in Europe may have Turkish staff on
/// English machines. Following the system alone would leave both of them with a
/// menu in the wrong language and nothing to do about it.
pub fn resolve(stored: Option<&str>) -> Lang {
    stored.and_then(Lang::from_code).unwrap_or_else(detect)
}

/// Turkish for a Turkish machine, English for every other. The product is sold
/// in Türkiye first and in Europe in parallel, and English is the fallback the
/// rest of the world reads.
pub fn detect() -> Lang {
    from_locale(sys_locale::get_locale().as_deref())
}

fn from_locale(locale: Option<&str>) -> Lang {
    match locale {
        Some(tag) if tag.to_ascii_lowercase().starts_with("tr") => Lang::Tr,
        _ => Lang::En,
    }
}

/// Some of these are drawn on one platform and not another: the application
/// menu, hiding and full screen are macOS's, the Help menu is everybody else's.
/// The struct carries all of them so that a translation is never platform
/// -specific, which is why the compiler is told not to count the unused ones.
#[allow(dead_code)]
pub struct Strings {
    pub menu_language: &'static str,
    pub lang_system: &'static str,
    pub menu_app: &'static str,
    pub menu_file: &'static str,
    pub menu_edit: &'static str,
    pub menu_view: &'static str,
    pub menu_help: &'static str,
    pub server: &'static str,
    pub downloads_folder: &'static str,
    pub check_updates: &'static str,
    pub reload: &'static str,
    pub fullscreen: &'static str,
    pub about: &'static str,
    pub quit: &'static str,
    pub hide: &'static str,
    pub hide_others: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub cut: &'static str,
    pub copy: &'static str,
    pub paste: &'static str,
    pub select_all: &'static str,
    pub minimize: &'static str,
    pub close: &'static str,
    pub saved_title: &'static str,
    pub saved_body: &'static str,
    pub update_failed_title: &'static str,
}

const TR: Strings = Strings {
    menu_language: "Dil",
    lang_system: "Sistemin dili",
    menu_app: "WaterForm",
    menu_file: "Dosya",
    menu_edit: "Düzen",
    menu_view: "Görünüm",
    menu_help: "Yardım",
    server: "Sunucu…",
    downloads_folder: "İndirilenler klasörünü aç",
    check_updates: "Güncellemeleri denetle…",
    reload: "Yeniden yükle",
    fullscreen: "Tam ekran",
    about: "WaterForm hakkında",
    quit: "WaterForm'dan çık",
    hide: "WaterForm'u gizle",
    hide_others: "Diğerlerini gizle",
    undo: "Geri al",
    redo: "Yinele",
    cut: "Kes",
    copy: "Kopyala",
    paste: "Yapıştır",
    select_all: "Tümünü seç",
    minimize: "Küçült",
    close: "Kapat",
    saved_title: "İndirildi",
    saved_body: "İndirilenler klasörüne kaydedildi.",
    update_failed_title: "Güncelleme denetlenemedi",
};

const EN: Strings = Strings {
    menu_language: "Language",
    lang_system: "System language",
    menu_app: "WaterForm",
    menu_file: "File",
    menu_edit: "Edit",
    menu_view: "View",
    menu_help: "Help",
    server: "Server…",
    downloads_folder: "Open downloads folder",
    check_updates: "Check for updates…",
    reload: "Reload",
    fullscreen: "Full screen",
    about: "About WaterForm",
    quit: "Quit WaterForm",
    hide: "Hide WaterForm",
    hide_others: "Hide others",
    undo: "Undo",
    redo: "Redo",
    cut: "Cut",
    copy: "Copy",
    paste: "Paste",
    select_all: "Select all",
    minimize: "Minimise",
    close: "Close",
    saved_title: "Downloaded",
    saved_body: "Saved to your downloads folder.",
    update_failed_title: "Could not check for updates",
};

pub fn strings(lang: Lang) -> &'static Strings {
    match lang {
        Lang::Tr => &TR,
        Lang::En => &EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turkish_machine_gets_turkish() {
        assert_eq!(from_locale(Some("tr-TR")), Lang::Tr);
        assert_eq!(from_locale(Some("tr")), Lang::Tr);
        assert_eq!(from_locale(Some("TR-tr")), Lang::Tr);
    }

    #[test]
    fn a_choice_beats_the_machine() {
        assert_eq!(resolve(Some("tr")), Lang::Tr);
        assert_eq!(resolve(Some("en")), Lang::En);
        // Nonsense in the settings file is not a language; the machine decides.
        assert_eq!(resolve(Some("de")), detect());
        assert_eq!(resolve(None), detect());
    }

    #[test]
    fn everything_else_gets_english() {
        assert_eq!(from_locale(Some("en-GB")), Lang::En);
        assert_eq!(from_locale(Some("de-DE")), Lang::En);
        assert_eq!(from_locale(None), Lang::En);
        // Turkmen is not Turkish.
        assert_eq!(from_locale(Some("tk-TM")), Lang::En);
    }
}
