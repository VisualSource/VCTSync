use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::LibError;

/// Preloader file names, in the order r2modman picks them.
/// `BepInEx.Preloader.dll` is the BepInEx 5 (Void Crew) case.
const PRELOADER_NAMES: [&str; 5] = [
    "BepInEx.Unity.Mono.Preloader.dll",
    "BepInEx.Unity.IL2CPP.dll",
    "BepInEx.Preloader.dll",
    "BepInEx.IL2CPP.dll",
    "BepInEx.NET.CoreCLR.dll",
];

/// An isolated BepInEx profile directory.
///
/// BepInEx derives its whole tree from the preloader location
/// (`<root>/BepInEx/core/<preloader>.dll`), so plugins, config and logs
/// all live here instead of in the game folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    root: PathBuf,
}

impl Profile {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn bepinex_dir(&self) -> PathBuf {
        self.root.join("BepInEx")
    }

    pub fn core_dir(&self) -> PathBuf {
        self.bepinex_dir().join("core")
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.bepinex_dir().join("plugins")
    }

    pub fn config_dir(&self) -> PathBuf {
        self.bepinex_dir().join("config")
    }

    pub fn patchers_dir(&self) -> PathBuf {
        self.bepinex_dir().join("patchers")
    }

    /// Written by BepInEx on launch; its presence confirms the profile redirect worked.
    pub fn log_file(&self) -> PathBuf {
        self.bepinex_dir().join("LogOutput.log")
    }

    pub fn ensure_layout(&self) -> Result<(), LibError> {
        fs::create_dir_all(self.plugins_dir())?;
        fs::create_dir_all(self.patchers_dir())?;
        Ok(())
    }

    /// Absolute path to the preloader dll doorstop should invoke.
    ///
    /// Uses `path::absolute` rather than `canonicalize` so windows paths
    /// don't get the `\\?\` verbatim prefix.
    pub fn find_preloader(&self) -> Result<PathBuf, LibError> {
        let core = self.core_dir();

        let found = PRELOADER_NAMES
            .iter()
            .map(|name| core.join(name))
            .find(|path| path.is_file())
            .ok_or_else(|| LibError::PreloaderNotFound(core.clone()))?;

        Ok(std::path::absolute(found)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preloader_missing() {
        let dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(dir.path());

        assert!(matches!(
            profile.find_preloader(),
            Err(LibError::PreloaderNotFound(_))
        ));
    }

    #[test]
    fn preloader_priority() {
        let dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(dir.path());
        fs::create_dir_all(profile.core_dir()).unwrap();
        fs::write(profile.core_dir().join("BepInEx.IL2CPP.dll"), "").unwrap();
        fs::write(profile.core_dir().join("BepInEx.Preloader.dll"), "").unwrap();

        let found = profile.find_preloader().unwrap();
        assert!(found.is_absolute());
        assert_eq!(found.file_name().unwrap(), "BepInEx.Preloader.dll");
    }

    #[test]
    fn ensure_layout_creates_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(dir.path());
        profile.ensure_layout().unwrap();

        assert!(profile.plugins_dir().is_dir());
        assert!(profile.patchers_dir().is_dir());
    }
}
