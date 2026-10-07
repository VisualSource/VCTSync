use std::{ffi::OsString, fs, path::Path};

use crate::error::LibError;

pub const DOORSTOP_VERSION_FILE: &str = ".doorstop_version";
pub const DOORSTOP_CONFIG_FILE: &str = "doorstop_config.ini";
pub const DOORSTOP_PROXY_DLL: &str = "winhttp.dll";

/// UnityDoorstop major version; the CLI arg names changed between 3 and 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorstopVersion {
    V3,
    V4,
}

impl DoorstopVersion {
    /// Read `.doorstop_version` from `dir`. Like r2modman, anything missing
    /// or unparseable is treated as v3.
    pub fn detect(dir: &Path) -> Self {
        fs::read_to_string(dir.join(DOORSTOP_VERSION_FILE))
            .map(|data| Self::parse(&data))
            .unwrap_or(Self::V3)
    }

    pub fn parse(data: &str) -> Self {
        let major = data
            .trim()
            .split('.')
            .next()
            .and_then(|major| major.parse::<u32>().ok());

        match major {
            Some(major) if major > 3 => Self::V4,
            _ => Self::V3,
        }
    }

    fn enabled_arg(self) -> &'static str {
        match self {
            Self::V3 => "--doorstop-enable",
            Self::V4 => "--doorstop-enabled",
        }
    }

    fn target_arg(self) -> &'static str {
        match self {
            Self::V3 => "--doorstop-target",
            Self::V4 => "--doorstop-target-assembly",
        }
    }

    /// Args that make doorstop load the given preloader.
    pub fn modded_args(self, preloader: &Path) -> Vec<OsString> {
        vec![
            self.enabled_arg().into(),
            "true".into(),
            self.target_arg().into(),
            preloader.as_os_str().to_owned(),
        ]
    }

    /// Args that disable doorstop for a vanilla launch.
    pub fn vanilla_args(self) -> Vec<OsString> {
        vec![self.enabled_arg().into(), "false".into()]
    }
}

/// The `[General]` section of a Doorstop 4 `doorstop_config.ini`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoorstopConfig {
    pub enabled: bool,
    pub target_assembly: Option<String>,
    pub redirect_output_log: bool,
    pub ignore_disable_switch: bool,
}

impl Default for DoorstopConfig {
    /// Defaults doorstop applies to keys missing from an existing ini.
    fn default() -> Self {
        Self {
            enabled: true,
            target_assembly: None,
            redirect_output_log: false,
            ignore_disable_switch: false,
        }
    }
}

impl DoorstopConfig {
    /// Read `doorstop_config.ini` from `dir`. `None` if it doesn't exist, in
    /// which case doorstop is disabled unless `--doorstop-enabled true` is passed.
    pub fn read(dir: &Path) -> Result<Option<Self>, LibError> {
        let path = dir.join(DOORSTOP_CONFIG_FILE);
        if !path.is_file() {
            return Ok(None);
        }

        Ok(Some(Self::parse(&fs::read_to_string(path)?)))
    }

    pub fn parse(data: &str) -> Self {
        let mut config = Self::default();
        let mut in_general = false;

        for line in data.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some(section) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                in_general = section.trim().eq_ignore_ascii_case("General");
                continue;
            }

            if !in_general {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();

            match key.trim() {
                "enabled" => config.enabled = parse_bool(value, config.enabled),
                "redirect_output_log" => {
                    config.redirect_output_log = parse_bool(value, config.redirect_output_log)
                }
                "ignore_disable_switch" => {
                    config.ignore_disable_switch = parse_bool(value, config.ignore_disable_switch)
                }
                "target_assembly" => {
                    config.target_assembly = (!value.is_empty()).then(|| value.to_owned())
                }
                _ => {}
            }
        }

        config
    }
}

/// Doorstop only accepts `true`/`false` (case-insensitive); anything else keeps the previous value.
fn parse_bool(value: &str, previous: bool) -> bool {
    if value.eq_ignore_ascii_case("true") {
        true
    } else if value.eq_ignore_ascii_case("false") {
        false
    } else {
        previous
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACK_INI: &str = "[General]
enabled = true
target_assembly=BepInEx\\core\\BepInEx.Preloader.dll
redirect_output_log = false
boot_config_override =
ignore_disable_switch = false
[UnityMono]
dll_search_path_override =
debug_enabled = false
debug_address = 127.0.0.1:10000
debug_suspend = false
";

    #[test]
    fn version_parse() {
        assert_eq!(DoorstopVersion::parse("4.5.0\n"), DoorstopVersion::V4);
        assert_eq!(DoorstopVersion::parse("3.4.0.0"), DoorstopVersion::V3);
        assert_eq!(DoorstopVersion::parse("garbage"), DoorstopVersion::V3);
    }

    #[test]
    fn version_detect_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(DoorstopVersion::detect(dir.path()), DoorstopVersion::V3);

        fs::write(dir.path().join(DOORSTOP_VERSION_FILE), "4.5.0").unwrap();
        assert_eq!(DoorstopVersion::detect(dir.path()), DoorstopVersion::V4);
    }

    #[test]
    fn args() {
        let preloader = Path::new("C:\\profile\\BepInEx\\core\\BepInEx.Preloader.dll");

        assert_eq!(
            DoorstopVersion::V4.modded_args(preloader),
            [
                "--doorstop-enabled",
                "true",
                "--doorstop-target-assembly",
                "C:\\profile\\BepInEx\\core\\BepInEx.Preloader.dll"
            ]
        );
        assert_eq!(
            DoorstopVersion::V3.modded_args(preloader),
            [
                "--doorstop-enable",
                "true",
                "--doorstop-target",
                "C:\\profile\\BepInEx\\core\\BepInEx.Preloader.dll"
            ]
        );
        assert_eq!(
            DoorstopVersion::V4.vanilla_args(),
            ["--doorstop-enabled", "false"]
        );
        assert_eq!(
            DoorstopVersion::V3.vanilla_args(),
            ["--doorstop-enable", "false"]
        );
    }

    #[test]
    fn config_parse_pack_ini() {
        let config = DoorstopConfig::parse(PACK_INI);
        assert_eq!(
            config,
            DoorstopConfig {
                enabled: true,
                target_assembly: Some("BepInEx\\core\\BepInEx.Preloader.dll".into()),
                redirect_output_log: false,
                ignore_disable_switch: false,
            }
        );
    }

    #[test]
    fn config_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(DoorstopConfig::read(dir.path()).unwrap(), None);
    }
}
