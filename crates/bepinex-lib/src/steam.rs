use std::path::{Path, PathBuf};

use crate::{VOID_CREW_APP_ID, VOID_CREW_EXE, error::LibError};

/// Location of the Steam client and the Void Crew install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamPaths {
    pub steam_dir: PathBuf,
    pub game_dir: PathBuf,
}

impl SteamPaths {
    /// Use explicit paths, e.g. ones the user overrode in settings.
    pub fn new(steam_dir: impl Into<PathBuf>, game_dir: impl Into<PathBuf>) -> Self {
        Self {
            steam_dir: steam_dir.into(),
            game_dir: game_dir.into(),
        }
    }

    /// Find Steam (registry on windows) and the library that holds Void Crew.
    pub fn locate() -> Result<Self, LibError> {
        let steam = steamlocate::SteamDir::locate().map_err(|_| LibError::SteamNotFound)?;

        let (app, library) = steam
            .find_app(VOID_CREW_APP_ID)?
            .ok_or(LibError::GameNotFound)?;

        let game_dir = library.resolve_app_dir(&app);

        if !game_dir.join(VOID_CREW_EXE).is_file() {
            return Err(LibError::GameNotFound);
        }

        Ok(Self {
            steam_dir: steam.path().to_path_buf(),
            game_dir,
        })
    }

    pub fn steam_exe(&self) -> PathBuf {
        self.steam_dir.join("Steam.exe")
    }

    pub fn game_exe(&self) -> PathBuf {
        self.game_dir.join(VOID_CREW_EXE)
    }

    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }
}
