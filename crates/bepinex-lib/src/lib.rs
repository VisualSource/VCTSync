//! Install and launch a BepInEx-modded Void Crew from an isolated profile directory.
//!
//! BepInEx derives its whole folder tree (`plugins`, `config`, `patchers`,
//! `LogOutput.log`) from the preloader dll that Doorstop is told to load. Pointing
//! Doorstop at `<profile>/BepInEx/core/BepInEx.Preloader.dll` keeps all mods in the
//! profile; only the Doorstop proxy files (`winhttp.dll`, `doorstop_config.ini`,
//! `.doorstop_version`) have to be copied next to the game exe.
//! See `docs/bepinex-launch-research.md` for the details.
//!
//! # Usage
//!
//! 1. Locate Steam and the game ([`SteamPaths::locate`], or [`SteamPaths::new`] for
//!    user-provided paths).
//! 2. Install the BepInExPack into a profile ([`install_bepinex_pack`]). Mods go in
//!    [`Profile::plugins_dir`].
//! 3. Copy the Doorstop files into the game folder ([`link_profile`]). Re-running it is
//!    cheap; unchanged files are skipped.
//! 4. Launch ([`Launcher::launch`]). Launching is Windows only; on other platforms it
//!    returns [`LibError::UnsupportedPlatform`].
//!
//! ```no_run
//! use bepinex_lib::{
//!     DEFAULT_BEPINEX_PACK_VERSION, LaunchMethod, LaunchMode, Launcher, Profile, SteamPaths,
//!     install_bepinex_pack, link_profile,
//! };
//!
//! # async fn run() -> Result<(), bepinex_lib::LibError> {
//! let http = reqwest::Client::new();
//! let steam = SteamPaths::locate()?;
//! let profile = Profile::new(r"C:\Users\me\AppData\Roaming\vct-sync\profiles\default");
//!
//! // one-time setup
//! install_bepinex_pack(&http, DEFAULT_BEPINEX_PACK_VERSION, &profile).await?;
//!
//! // before every modded launch (the profile may have changed)
//! link_profile(&profile, steam.game_dir())?;
//!
//! let launcher = Launcher::new(steam, profile);
//! launcher.launch(LaunchMode::Modded, LaunchMethod::Steam, None::<String>)?;
//!
//! // vanilla: doorstop is explicitly disabled, the profile is ignored
//! launcher.launch(LaunchMode::Vanilla, LaunchMethod::Steam, None::<String>)?;
//! # Ok(())
//! # }
//! ```
//!
//! Use [`Launcher::build_command`] to inspect the command without running it.
//! If BepInEx doesn't load, check whether [`Profile::log_file`] was created; if it
//! wasn't, Doorstop never ran or was pointed somewhere else.
//! To fully restore the game folder, call [`unlink_profile`].

mod doorstop;
mod error;
mod install;
mod launch;
mod link;
mod profile;
mod steam;

pub use doorstop::{DoorstopConfig, DoorstopVersion};
pub use error::LibError;
pub use install::{DEFAULT_BEPINEX_PACK_VERSION, install_bepinex_pack, install_build};
pub use launch::{LaunchMethod, LaunchMode, Launcher};
pub use link::{link_profile, unlink_profile};
pub use profile::Profile;
pub use steam::SteamPaths;

pub const VOID_CREW_APP_ID: u32 = 1063420;
pub const VOID_CREW_EXE: &str = "Void Crew.exe";
