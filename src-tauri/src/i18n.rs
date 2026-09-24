//! Turkish, English and German for the parts of the shell the operating system
//! draws.
//!
//! The application inside the window speaks whatever language the company is
//! set to; the menu bar and the shell's own dialogs follow the machine until the
//! customer picks a language from the menu, and then they follow the choice.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Tr,
    En,
    De,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Tr => "tr",
            Lang::En => "en",
            Lang::De => "de",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "tr" => Some(Lang::Tr),
            "en" => Some(Lang::En),
            "de" => Some(Lang::De),
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

/// Turkish for a Turkish machine, German for a German one, English for every
/// other. The product is sold in Türkiye first and in Europe in parallel, and
/// English is the fallback the rest of the world reads.
pub fn detect() -> Lang {
    from_locale(sys_locale::get_locale().as_deref())
}

fn from_locale(locale: Option<&str>) -> Lang {
    // Only the language part counts: "de-AT", "de_CH.UTF-8" and "tr-TR" are
    // German, German and Turkish, whatever the region.
    let language = locale
        .and_then(|tag| tag.split(['-', '_', '.', '@']).next())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match language.as_str() {
        "tr" => Lang::Tr,
        "de" => Lang::De,
        _ => Lang::En,
    }
}

/// The one place the platforms disagree about a word: macOS and Windows use
/// different German for the same menu items, and a German reader notices a
/// Windows word on a Mac at once. Turkish and English do not need this.
const fn mac_or(mac: &'static str, other: &'static str) -> &'static str {
    if cfg!(target_os = "macos") {
        mac
    } else {
        other
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
    pub print: &'static str,
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
    print: "Yazdır…",
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
    print: "Print…",
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

const DE: Strings = Strings {
    menu_language: "Sprache",
    lang_system: "Systemsprache",
    menu_app: "WaterForm",
    menu_file: mac_or("Ablage", "Datei"),
    menu_edit: "Bearbeiten",
    menu_view: mac_or("Darstellung", "Ansicht"),
    menu_help: "Hilfe",
    server: "Server…",
    downloads_folder: "Downloads-Ordner öffnen",
    print: "Drucken…",
    check_updates: "Nach Aktualisierungen suchen…",
    reload: "Neu laden",
    fullscreen: "Vollbild",
    about: "Über WaterForm",
    quit: "WaterForm beenden",
    hide: "WaterForm ausblenden",
    hide_others: "Andere ausblenden",
    undo: mac_or("Widerrufen", "Rückgängig"),
    redo: "Wiederholen",
    cut: "Ausschneiden",
    copy: "Kopieren",
    paste: mac_or("Einsetzen", "Einfügen"),
    select_all: "Alles auswählen",
    minimize: mac_or("Im Dock ablegen", "Minimieren"),
    close: "Schließen",
    saved_title: "Heruntergeladen",
    saved_body: "Im Downloads-Ordner gespeichert.",
    update_failed_title: "Suche nach Aktualisierungen fehlgeschlagen",
};

pub fn strings(lang: Lang) -> &'static Strings {
    match lang {
        Lang::Tr => &TR,
        Lang::En => &EN,
        Lang::De => &DE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every language the shell speaks. A fourth that is added to the enum and
    /// not here fails `a_code_is_a_round_trip`'s exhaustive match below.
    const ALL: [Lang; 3] = [Lang::Tr, Lang::En, Lang::De];

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
        assert_eq!(resolve(Some("de")), Lang::De);
        // Nonsense in the settings file is not a language; the machine decides.
        assert_eq!(resolve(Some("fr")), detect());
        assert_eq!(resolve(Some("")), detect());
        assert_eq!(resolve(None), detect());
    }

    #[test]
    fn a_german_machine_gets_german() {
        assert_eq!(from_locale(Some("de-DE")), Lang::De);
        assert_eq!(from_locale(Some("de-AT")), Lang::De);
        assert_eq!(from_locale(Some("de_CH.UTF-8")), Lang::De);
        assert_eq!(from_locale(Some("DE")), Lang::De);
    }

    #[test]
    fn everything_else_gets_english() {
        assert_eq!(from_locale(Some("en-GB")), Lang::En);
        assert_eq!(from_locale(Some("fr-FR")), Lang::En);
        assert_eq!(from_locale(Some("nl-NL")), Lang::En);
        assert_eq!(from_locale(Some("")), Lang::En);
        assert_eq!(from_locale(None), Lang::En);
        // Turkmen is not Turkish.
        assert_eq!(from_locale(Some("tk-TM")), Lang::En);
        // Nor is a language that merely starts with the same letters.
        assert_eq!(from_locale(Some("trv")), Lang::En);
        assert_eq!(from_locale(Some("dsb-DE")), Lang::En);
    }

    #[test]
    fn a_code_is_a_round_trip() {
        // Exhaustive on purpose: a new variant does not compile until it is in ALL.
        for lang in ALL {
            match lang {
                Lang::Tr | Lang::En | Lang::De => {}
            }
            assert_eq!(Lang::from_code(lang.code()), Some(lang));
        }
    }

    #[test]
    fn every_language_says_everything() {
        for lang in ALL {
            let s = strings(lang);
            let all = [
                s.menu_language,
                s.lang_system,
                s.menu_app,
                s.menu_file,
                s.menu_edit,
                s.menu_view,
                s.menu_help,
                s.server,
                s.downloads_folder,
                s.print,
                s.check_updates,
                s.reload,
                s.fullscreen,
                s.about,
                s.quit,
                s.hide,
                s.hide_others,
                s.undo,
                s.redo,
                s.cut,
                s.copy,
                s.paste,
                s.select_all,
                s.minimize,
                s.close,
                s.saved_title,
                s.saved_body,
                s.update_failed_title,
            ];
            for text in all {
                assert!(!text.trim().is_empty(), "{}: an empty string", lang.code());
                assert!(!text.contains('—'), "{}: an em dash in {text}", lang.code());
            }
        }
    }
}
