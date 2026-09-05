//! Shared types and logic for `vct-agent` (headless, Linux) and `vct-sync` (GUI, Windows).
//!
//! Nothing in here pulls in `axum` or `iced` — it is the seam between the two ends.

pub mod build;
pub mod config;
pub mod github;
pub mod http;
pub mod install;
pub mod manifest;
pub mod protocol;

/// Default TCP port the agent binds to.
pub const DEFAULT_PORT: u16 = 9787;

/// Thunderstore package identity: `<Author>-<Package>`. This is the folder name
/// under `BepInEx/plugins/` and the `name` field in `mods.yml` — TMM throws
/// "Path undefined" if it is missing.
pub const PACKAGE_NAME: &str = "VisualSource-VoidCrewTerminus";
/// Bare mod name — the `displayName` in `mods.yml`, and the assembly / DLL stem.
pub const MOD_NAME: &str = "VoidCrewTerminus";
pub const MOD_AUTHOR: &str = "VisualSource";
pub const GITHUB_REPO: &str = "VisualSource/VoidCrewTerminus";
/// Substring that tags this mod's lines in `LogOutput.log`
/// (e.g. `[Info   :VoidCrewTerminus] …`). Used for the "mod lines only" filter.
pub const MOD_LOG_TAG: &str = "VoidCrew";
