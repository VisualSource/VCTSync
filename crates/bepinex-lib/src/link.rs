use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    doorstop::{DOORSTOP_CONFIG_FILE, DOORSTOP_PROXY_DLL, DOORSTOP_VERSION_FILE},
    error::LibError,
    profile::Profile,
};

/// Profile root files that never go into the game folder.
const LINK_EXCLUDE: [&str; 1] = ["mods.yml"];

/// Copy the profile's root files (`winhttp.dll`, `doorstop_config.ini`,
/// `.doorstop_version`, ...) into the game folder. Doorstop is a proxy dll and
/// must sit next to the exe. Directories are never copied, so `BepInEx/`
/// stays in the profile.
///
/// Returns the paths written; files that already match by size and mtime are skipped.
pub fn link_profile(profile: &Profile, game_dir: &Path) -> Result<Vec<PathBuf>, LibError> {
    let mut linked = Vec::new();

    for entry in fs::read_dir(profile.root())? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let name = entry.file_name();
        if LINK_EXCLUDE
            .iter()
            .any(|ex| name.to_string_lossy().eq_ignore_ascii_case(ex))
        {
            continue;
        }

        let from = entry.path();
        let to = game_dir.join(&name);

        if is_identical(&from, &to)? {
            continue;
        }

        fs::copy(&from, &to)?;
        let modified = fs::metadata(&from)?.modified()?;
        fs::File::options()
            .write(true)
            .open(&to)?
            .set_modified(modified)?;

        linked.push(to);
    }

    Ok(linked)
}

/// Remove the doorstop files from the game folder so it is fully vanilla.
pub fn unlink_profile(game_dir: &Path) -> Result<(), LibError> {
    for name in [DOORSTOP_PROXY_DLL, DOORSTOP_CONFIG_FILE, DOORSTOP_VERSION_FILE] {
        let path = game_dir.join(name);
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn is_identical(from: &Path, to: &Path) -> Result<bool, LibError> {
    let Ok(to_meta) = fs::metadata(to) else {
        return Ok(false);
    };
    let from_meta = fs::metadata(from)?;

    Ok(from_meta.len() == to_meta.len() && from_meta.modified()? == to_meta.modified()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_root_files_only() {
        let profile_dir = tempfile::tempdir().unwrap();
        let game_dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(profile_dir.path());

        fs::write(profile_dir.path().join(DOORSTOP_PROXY_DLL), "dll").unwrap();
        fs::write(profile_dir.path().join(DOORSTOP_CONFIG_FILE), "ini").unwrap();
        fs::write(profile_dir.path().join(DOORSTOP_VERSION_FILE), "4.5.0").unwrap();
        fs::write(profile_dir.path().join("mods.yml"), "").unwrap();
        fs::create_dir_all(profile.core_dir()).unwrap();
        fs::write(profile.core_dir().join("BepInEx.Preloader.dll"), "").unwrap();

        let linked = link_profile(&profile, game_dir.path()).unwrap();
        assert_eq!(linked.len(), 3);
        assert!(game_dir.path().join(DOORSTOP_PROXY_DLL).is_file());
        assert!(game_dir.path().join(DOORSTOP_CONFIG_FILE).is_file());
        assert!(game_dir.path().join(DOORSTOP_VERSION_FILE).is_file());
        assert!(!game_dir.path().join("mods.yml").exists());
        assert!(!game_dir.path().join("BepInEx").exists());

        // second link is a no-op
        assert!(link_profile(&profile, game_dir.path()).unwrap().is_empty());

        unlink_profile(game_dir.path()).unwrap();
        assert_eq!(fs::read_dir(game_dir.path()).unwrap().count(), 0);
    }
}
