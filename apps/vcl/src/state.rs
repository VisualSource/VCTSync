use nanoserde::{DeJson, DeRon, SerJson, SerRon};

#[derive(Clone)]
pub enum Message {
    Js(iced_js::Event),
    Reload,
}

/// Stored in `<app dir>/settings.ron`; keys match the view's `Settings` (view/state.tsx).
/// Empty strings mean "default" so older or partial files still load.
#[derive(Debug, Clone, Default, DeJson, SerJson, SerRon, DeRon)]
pub struct AppSettings {
    /// An iced theme name (`Theme::ALL`); empty is Dark.
    #[nserde(default)]
    pub theme: String,
    /// Steam folder override; empty means auto-detect.
    #[nserde(default)]
    #[nserde(rename = "steamDir")]
    pub steam_dir: String,
    /// "steam" or "direct"; empty is steam.
    #[nserde(default)]
    #[nserde(rename = "launchMethod")]
    pub launch_method: String,
    #[nserde(default)]
    #[nserde(rename = "modOnlyLogLines")]
    pub mod_only_log_lines: bool,
    #[nserde(default)]
    #[nserde(rename = "devApiUrl")]
    pub dev_api_url: String,
    #[nserde(default)]
    #[nserde(rename = "devApiToken")]
    pub dev_api_token: String,
    #[nserde(default)]
    #[nserde(rename = "saveDir")]
    pub save_dir: String,
}

impl AppSettings {
    /// Load `settings.ron`, falling back to defaults (and logging why) if it's missing or invalid.
    pub fn load(file: &std::path::Path) -> Self {
        match std::fs::read_to_string(file) {
            Ok(ron) => AppSettings::deserialize_ron(&ron).unwrap_or_else(|err| {
                log::error!("invalid {}: {err}", file.display());
                Self::default()
            }),
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    log::error!("reading {}: {err}", file.display());
                }
                Self::default()
            }
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::ALL
            .iter()
            .find(|theme| theme.to_string() == self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Dark)
    }

    pub fn steam(&self) -> Result<bepinex_lib::SteamPaths, bepinex_lib::LibError> {
        if self.steam_dir.is_empty() {
            bepinex_lib::SteamPaths::locate()
        } else {
            bepinex_lib::SteamPaths::locate_in(std::path::Path::new(&self.steam_dir))
        }
    }

    pub fn launch_method(&self) -> bepinex_lib::LaunchMethod {
        match self.launch_method.as_str() {
            "direct" => bepinex_lib::LaunchMethod::Direct,
            _ => bepinex_lib::LaunchMethod::Steam,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_defaults_and_round_trip() {
        // The old empty settings.ron and a view payload that only sets some keys.
        let empty = AppSettings::deserialize_ron("()").unwrap();
        assert_eq!(empty.theme(), iced::Theme::Dark);
        assert!(matches!(empty.launch_method(), bepinex_lib::LaunchMethod::Steam));

        let from_view = AppSettings::deserialize_json(r#"{"theme":"Nord","launchMethod":"direct","devApiUrl":"10.0.0.2:9787","extra":1}"#).unwrap();
        assert_eq!(from_view.theme(), iced::Theme::Nord);
        assert!(matches!(from_view.launch_method(), bepinex_lib::LaunchMethod::Direct));

        let back = AppSettings::deserialize_ron(&from_view.serialize_ron()).unwrap();
        assert_eq!(back.dev_api_url, "10.0.0.2:9787");
        assert!(from_view.serialize_json().contains("\"devApiUrl\""));

        let unknown = AppSettings { theme: "Not A Theme".into(), ..Default::default() };
        assert_eq!(unknown.theme(), iced::Theme::Dark);
    }
}
