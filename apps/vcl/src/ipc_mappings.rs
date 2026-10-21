use nanoserde::{DeJson, DeRon, SerJson, SerRon};

#[derive(Debug, Default, DeJson)]
pub struct MessageDialogConfig {
    pub title: String,
    pub description: String,
    pub buttons: String,
    pub level: Option<String>,
}

#[derive(SerJson)]
pub struct MessageDialogResult {
    pub result: String,
}

impl MessageDialogResult {
    pub fn new(value: String) -> Self {
        Self { result: value }
    }
}

#[derive(Debug, SerJson)]
pub struct OpenDialogResult {
    pub path: Option<String>,
}

impl OpenDialogResult {
    pub fn new(path: Option<String>) -> Self {
        Self { path }
    }
}

#[derive(Debug, Default, DeJson)]
pub struct OpenFileConfig {
    #[nserde(default)]
    pub filters: Vec<String>,
}

#[derive(Debug, DeJson)]
pub struct ZipPathConfig {
    pub path: String,
}

/// Install one mod into a profile, from a local `path` or a download `url`.
#[derive(Debug, DeJson)]
pub struct InstallModConfig {
    pub profile_dir: String,
    /// "Author-Name"
    pub full_name: String,
    pub path: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, DeJson)]
pub struct LaunchGameConfig {
    pub profile_dir: String,
    #[nserde(default)]
    pub launch_args: String,
}

#[derive(SerJson)]
pub struct InstallModResult {
    pub icon: Option<String>,
}

#[derive(Debug, DeJson)]
pub struct UninstallModConfig {
    pub profile_dir: String,
    /// "Author-Name"
    pub full_name: String,
}

#[derive(Debug, DeJson)]
pub struct SetModEnabledConfig {
    pub profile_dir: String,
    /// "Author-Name"
    pub full_name: String,
    pub enabled: bool,
}

#[derive(Debug, DeJson)]
pub struct ProfileDirConfig {
    pub profile_dir: String,
}

/// `deleted` is false when the folder was left alone because it is outside the app folder.
#[derive(SerJson)]
pub struct DeleteDirResult {
    pub deleted: bool,
    pub dir: String,
}

/// Mirrors `Mod` in view/pages/profiles/types.ts. Parsing into this drops anything else the
/// view sends (TanStack DB adds `$synced`, `$key`, ... to rows).
#[derive(Debug, DeJson, SerJson, DeRon, SerRon)]
pub struct ModRecord {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub enabled: bool,
    #[nserde(default)]
    pub icon: String,
    #[nserde(default)]
    pub description: String,
    #[nserde(default)]
    pub dependencies: Vec<String>,
    #[nserde(default)]
    pub source: String,
}

/// Mirrors `Profile` in view/pages/profiles/types.ts; stored in profiles.ron.
#[derive(Debug, DeJson, SerJson, DeRon, SerRon)]
pub struct ProfileRecord {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub dir: String,
    #[nserde(default)]
    pub launch_args: String,
    pub mods: Vec<ModRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(SerJson)]
pub struct AppInfo {
    #[nserde(rename = "appDir")]
    pub app_dir: String,
    pub version: String,
    /// iced theme names, valid values for `AppSettings::theme`.
    pub themes: Vec<String>,
}

/// Result of locating Steam + Void Crew with the current settings.
#[derive(SerJson)]
pub struct GameStatus {
    #[nserde(rename = "steamDir")]
    pub steam_dir: Option<String>,
    #[nserde(rename = "gameDir")]
    pub game_dir: Option<String>,
    /// The doorstop proxy (BepInEx loader) is in the game folder.
    pub modded: bool,
    pub error: Option<String>,
}

#[derive(Debug, DeJson)]
pub struct OpenPathConfig {
    pub path: String,
}

#[derive(SerJson)]
pub struct IpcError {
    pub error: String,
}

#[derive(Debug, DeJson)]
pub struct InstallBuildConfig {
    id: String,
    version: String,
    source_url: String,
    content_type: String,
    timestamp: String,
    branch: String,
    digest: String,
}
