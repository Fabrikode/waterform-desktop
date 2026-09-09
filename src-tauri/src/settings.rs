//! The three things the shell remembers between runs.
//!
//! A plain JSON file in the application's configuration directory. There is no
//! database and no plugin: the file is small, a support engineer can read it
//! down the phone, and a corrupt one costs the customer a re-typed address
//! rather than a reinstall.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// The server this installation talks to. `None` on a machine where nobody
    /// has been through the first-run screen yet.
    pub server_url: Option<String>,
    /// What that server last said about itself, so the About box can answer
    /// "which version am I on" without a round trip.
    pub last_version: Option<String>,
    /// Unix seconds. An update offer the customer put off is not offered again
    /// until this passes. Never blocks, only delays.
    pub update_snoozed_until: Option<i64>,
}

impl Settings {
    pub fn path(config_dir: &Path) -> PathBuf {
        config_dir.join("settings.json")
    }

    /// Reads the file. A missing or unreadable one is not an error: it is a
    /// machine that has not been set up, which is the state the first-run screen
    /// exists for.
    pub fn load(config_dir: &Path) -> Self {
        let path = Self::path(config_dir);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match serde_json::from_str(&text) {
            Ok(settings) => settings,
            Err(error) => {
                log::warn!("settings.json unreadable, starting fresh: {error}");
                Self::default()
            }
        }
    }

    pub fn save(&self, config_dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(config_dir)?;
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(config_dir), text)
    }

    pub fn snoozing(&self, now: i64) -> bool {
        self.update_snoozed_until.is_some_and(|until| now < until)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "waterform-desktop-test-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_machine_with_no_file_has_no_server() {
        let dir = scratch("empty");
        assert_eq!(Settings::load(&dir), Settings::default());
    }

    #[test]
    fn what_is_written_is_what_comes_back() {
        let dir = scratch("roundtrip");
        let settings = Settings {
            server_url: Some("https://depo.sirket.com".into()),
            last_version: Some("0c74ba8".into()),
            update_snoozed_until: Some(1_800_000_000),
        };
        settings.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir), settings);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_corrupt_file_costs_the_address_not_the_install() {
        let dir = scratch("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(Settings::path(&dir), "{ this is not json").unwrap();
        assert_eq!(Settings::load(&dir), Settings::default());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_unknown_field_does_not_lose_the_known_ones() {
        // A settings file written by a newer build must still open in an older one.
        let dir = scratch("forward");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            Settings::path(&dir),
            r#"{"serverUrl":"https://depo.sirket.com","somethingLater":42}"#,
        )
        .unwrap();
        assert_eq!(
            Settings::load(&dir).server_url.as_deref(),
            Some("https://depo.sirket.com")
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_snooze_expires() {
        let mut settings = Settings::default();
        assert!(!settings.snoozing(1_000));
        settings.update_snoozed_until = Some(2_000);
        assert!(settings.snoozing(1_999));
        assert!(!settings.snoozing(2_000));
    }
}
