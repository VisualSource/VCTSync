use std::{
    fs,
    path::{Path, PathBuf},
};

use reqwest::{Response, Url};
use tokio::io::AsyncWriteExt;

use crate::{
    doorstop::{DOORSTOP_CONFIG_FILE, DOORSTOP_PROXY_DLL, DOORSTOP_VERSION_FILE},
    error::LibError,
    profile::Profile,
};

/// BepInExPack version every current Void Crew mod depends on.
pub const DEFAULT_BEPINEX_PACK_VERSION: &str = "5.4.2305";

/// Folder inside the Thunderstore BepInExPack zip that is hoisted into the profile.
const PACK_ROOT_FOLDER: &str = "BepInExPack";

/// Thunderstore package metadata, never copied into the profile.
const PACK_SKIP_FILES: [&str; 3] = ["manifest.json", "readme.md", "icon.png"];

pub async fn install_build<F>(
    http_client: &reqwest::Client,
    source: &str,
    out_dir: &Path,
    progress: F,
) -> Result<(), LibError>
where
    F: AsyncFnMut(u64, u64),
{
    let url = Url::parse(source).map_err(|err| LibError::UrlParse(err.to_string()))?;
    let resp = http_client.get(url).send().await?.error_for_status()?;

    download_zip(resp, out_dir, progress).await
}

/// Download the Thunderstore BepInExPack and hoist its contents into the profile.
pub async fn install_bepinex_pack(
    http_client: &reqwest::Client,
    version: &str,
    profile: &Profile,
) -> Result<(), LibError> {
    let url = format!("https://thunderstore.io/package/download/BepInEx/BepInExPack/{version}/");
    let resp = http_client.get(url).send().await?.error_for_status()?;

    let extract_dir = tempfile::tempdir()?;
    download_zip(resp, extract_dir.path(), async |_, _| {}).await?;

    let profile = profile.clone();
    tokio::task::spawn_blocking(move || {
        let pack_root = extract_dir.path().join(PACK_ROOT_FOLDER);
        let pack_root = if pack_root.is_dir() {
            pack_root
        } else {
            extract_dir.path().to_path_buf()
        };

        hoist_pack(&pack_root, profile.root())?;
        profile.ensure_layout()
    })
    .await
    .map_err(std::io::Error::other)?
}

/// Contents of the Thunderstore `manifest.json` at the root of a mod zip, BOM stripped.
pub async fn read_zip_manifest(zip: &Path) -> Result<String, LibError> {
    let mut reader = s_zip::AsyncStreamingZipReader::open(zip).await?;
    let bytes = reader.read_entry_by_name("manifest.json").await?;
    Ok(String::from_utf8_lossy(&bytes).trim_start_matches('\u{feff}').to_string())
}

/// Thunderstore package that is hoisted into the profile root instead of installed as a plugin.
const BEPINEX_PACK: &str = "BepInEx-BepInExPack";

/// Folders that r2modman routes into `BepInEx/<folder>/<Author-Name>/`.
const SCOPED_FOLDERS: [&str; 4] = ["plugins", "patchers", "core", "monomod"];

/// Install a Thunderstore mod zip into the profile. `full_name` is "Author-Name".
/// Returns the installed `icon.png`, if the package has one.
pub async fn install_mod_zip(zip: &Path, full_name: &str, profile: &Profile) -> Result<Option<PathBuf>, LibError> {
    let extract_dir = tempfile::tempdir()?;
    s_zip::AsyncStreamingZipReader::open(zip)
        .await?
        .extract_all(extract_dir.path())
        .await?;

    install_extracted(extract_dir, full_name, profile).await
}

/// Download a Thunderstore mod and install it into the profile. `full_name` is "Author-Name".
/// Returns the installed `icon.png`, if the package has one.
pub async fn install_mod_url(
    http_client: &reqwest::Client,
    url: &str,
    full_name: &str,
    profile: &Profile,
) -> Result<Option<PathBuf>, LibError> {
    let url = Url::parse(url).map_err(|err| LibError::UrlParse(err.to_string()))?;
    let resp = http_client.get(url).send().await?.error_for_status()?;

    let extract_dir = tempfile::tempdir()?;
    download_zip(resp, extract_dir.path(), async |_, _| {}).await?;

    install_extracted(extract_dir, full_name, profile).await
}

async fn install_extracted(
    extract_dir: tempfile::TempDir,
    full_name: &str,
    profile: &Profile,
) -> Result<Option<PathBuf>, LibError> {
    let full_name = full_name.to_string();
    let profile = profile.clone();

    tokio::task::spawn_blocking(move || {
        if full_name == BEPINEX_PACK {
            let pack_root = extract_dir.path().join(PACK_ROOT_FOLDER);
            let pack_root = if pack_root.is_dir() { pack_root } else { extract_dir.path().to_path_buf() };
            hoist_pack(&pack_root, profile.root())?;
        } else {
            install_mod_tree(extract_dir.path(), &full_name, &profile)?;
        }
        profile.ensure_layout()?;

        // Loose files, icon.png included, land in plugins/<Author-Name>.
        let icon = profile.plugins_dir().join(&full_name).join("icon.png");
        Ok(icon.is_file().then(|| std::path::absolute(icon)).transpose()?)
    })
    .await
    .map_err(std::io::Error::other)?
}

/// Route an extracted mod into the profile's BepInEx folders, replacing a previous install.
fn install_mod_tree(src: &Path, full_name: &str, profile: &Profile) -> Result<(), LibError> {
    remove_mod_tree(full_name, profile)?;
    route_entries(src, full_name, &profile.bepinex_dir())
}

fn remove_mod_tree(full_name: &str, profile: &Profile) -> Result<(), LibError> {
    for folder in SCOPED_FOLDERS {
        let old = profile.bepinex_dir().join(folder).join(full_name);
        if old.is_dir() {
            fs::remove_dir_all(old)?;
        }
    }
    Ok(())
}

/// Suffix that hides a disabled mod's files from BepInEx (r2modman convention).
const DISABLED_SUFFIX: &str = ".old";

/// Enable or disable an installed mod by renaming its files to/from `<name>.old`.
pub fn set_mod_enabled(full_name: &str, profile: &Profile, enabled: bool) -> Result<(), LibError> {
    if full_name == BEPINEX_PACK {
        return Err(std::io::Error::other("the BepInExPack is the mod loader and can't be disabled").into());
    }

    for folder in SCOPED_FOLDERS {
        let dir = profile.bepinex_dir().join(folder).join(full_name);
        if dir.is_dir() {
            rename_files(&dir, enabled)?;
        }
    }
    Ok(())
}

fn rename_files(dir: &Path, enabled: bool) -> Result<(), LibError> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            rename_files(&path, enabled)?;
            continue;
        }

        let name = path.to_string_lossy();
        let target = match (enabled, name.strip_suffix(DISABLED_SUFFIX)) {
            (true, Some(original)) => PathBuf::from(original),
            (false, None) => PathBuf::from(format!("{name}{DISABLED_SUFFIX}")),
            _ => continue,
        };
        fs::rename(&path, target)?;
    }
    Ok(())
}

/// Remove an installed mod's files from the profile. `full_name` is "Author-Name".
/// Configs in `BepInEx/config` are kept, like r2modman. For the BepInExPack this
/// removes the loader (`BepInEx/core` and the doorstop files), so modded launches
/// fail until it is reinstalled.
pub fn uninstall_mod(full_name: &str, profile: &Profile) -> Result<(), LibError> {
    if full_name != BEPINEX_PACK {
        return remove_mod_tree(full_name, profile);
    }

    if profile.core_dir().is_dir() {
        fs::remove_dir_all(profile.core_dir())?;
    }
    for name in [DOORSTOP_PROXY_DLL, DOORSTOP_CONFIG_FILE, DOORSTOP_VERSION_FILE] {
        let path = profile.root().join(name);
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn route_entries(dir: &Path, full_name: &str, bepinex: &Path) -> Result<(), LibError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let lower = name.to_string_lossy().to_lowercase();
        let is_dir = entry.file_type()?.is_dir();

        if is_dir && lower == "bepinex" {
            route_entries(&entry.path(), full_name, bepinex)?;
        } else if is_dir && lower == "config" {
            copy_dir(&entry.path(), &bepinex.join("config"))?;
        } else if is_dir && SCOPED_FOLDERS.contains(&lower.as_str()) {
            copy_dir(&entry.path(), &bepinex.join(&lower).join(full_name))?;
        } else {
            let to = bepinex.join("plugins").join(full_name);
            fs::create_dir_all(&to)?;
            if is_dir {
                copy_dir(&entry.path(), &to.join(&name))?;
            } else {
                fs::copy(entry.path(), to.join(&name))?;
            }
        }
    }

    Ok(())
}

/// Download a zip to a temp file and extract it into `out_dir`.
/// `progress` receives `(downloaded, total)`; `total` is 0 when unknown.
async fn download_zip<F>(mut resp: Response, out_dir: &Path, mut progress: F) -> Result<(), LibError>
where
    F: AsyncFnMut(u64, u64),
{
    let temp = tempfile::Builder::new()
        .suffix(".zip")
        .tempfile()?
        .into_temp_path();

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded = 0;
    let mut file = tokio::fs::File::create(&temp).await?;

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        progress(downloaded, total).await;
    }
    file.flush().await?;
    drop(file);

    let mut zip = s_zip::AsyncStreamingZipReader::open(&temp).await?;
    zip.extract_all(out_dir).await?;

    Ok(())
}

/// Recursively copy the pack contents into the profile root, skipping
/// Thunderstore metadata at the top level.
fn hoist_pack(pack_root: &Path, profile_root: &Path) -> Result<(), LibError> {
    fs::create_dir_all(profile_root)?;

    for entry in fs::read_dir(pack_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let lower = name.to_string_lossy().to_lowercase();

        if PACK_SKIP_FILES.contains(&lower.as_str()) {
            continue;
        }

        let to = profile_root.join(&name);
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }

    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), LibError> {
    fs::create_dir_all(to)?;

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disable_and_enable_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(dir.path());
        let plugin = profile.plugins_dir().join("Author-Mod");
        fs::create_dir_all(plugin.join("sub")).unwrap();
        fs::write(plugin.join("Mod.dll"), "x").unwrap();
        fs::write(plugin.join("sub/Extra.dll"), "x").unwrap();

        set_mod_enabled("Author-Mod", &profile, false).unwrap();
        assert!(plugin.join("Mod.dll.old").is_file());
        assert!(plugin.join("sub/Extra.dll.old").is_file());
        assert!(!plugin.join("Mod.dll").exists());

        // Disabling twice must not stack suffixes.
        set_mod_enabled("Author-Mod", &profile, false).unwrap();
        assert!(!plugin.join("Mod.dll.old.old").exists());

        set_mod_enabled("Author-Mod", &profile, true).unwrap();
        assert!(plugin.join("Mod.dll").is_file());
        assert!(plugin.join("sub/Extra.dll").is_file());

        assert!(set_mod_enabled(BEPINEX_PACK, &profile, false).is_err());
    }

    #[test]
    fn uninstall_removes_mod_keeps_config() {
        let dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(dir.path());
        fs::create_dir_all(profile.plugins_dir().join("Author-Mod")).unwrap();
        fs::create_dir_all(profile.plugins_dir().join("Other-Mod")).unwrap();
        fs::create_dir_all(profile.config_dir()).unwrap();
        fs::write(profile.config_dir().join("mod.cfg"), "x").unwrap();

        uninstall_mod("Author-Mod", &profile).unwrap();

        assert!(!profile.plugins_dir().join("Author-Mod").exists());
        assert!(profile.plugins_dir().join("Other-Mod").is_dir());
        assert!(profile.config_dir().join("mod.cfg").is_file());
        // Already gone is fine.
        uninstall_mod("Author-Mod", &profile).unwrap();
    }

    #[test]
    fn install_returns_icon() {
        let dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(dir.path().join("profile"));
        let zip = dir.path().join("Author-Mod-1.0.0.zip");
        let mut writer = s_zip::StreamingZipWriter::new(&zip).unwrap();
        writer.add_entry("icon.png", b"png").unwrap();
        writer.add_entry("Mod.dll", b"dll").unwrap();
        writer.finish().unwrap();

        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let icon = rt.block_on(install_mod_zip(&zip, "Author-Mod", &profile)).unwrap().unwrap();

        assert!(icon.is_absolute());
        assert_eq!(icon, std::path::absolute(profile.plugins_dir().join("Author-Mod/icon.png")).unwrap());
    }

    #[test]
    fn routes_mod_files() {
        let src = tempfile::tempdir().unwrap();
        let profile_dir = tempfile::tempdir().unwrap();
        let profile = Profile::new(profile_dir.path());
        let root = src.path();

        fs::write(root.join("Loose.dll"), "x").unwrap();
        fs::create_dir_all(root.join("config")).unwrap();
        fs::write(root.join("config/mod.cfg"), "x").unwrap();
        fs::create_dir_all(root.join("BepInEx/patchers")).unwrap();
        fs::write(root.join("BepInEx/patchers/Patch.dll"), "x").unwrap();

        let stale = profile.plugins_dir().join("Author-Mod/Old.dll");
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&stale, "x").unwrap();

        install_mod_tree(root, "Author-Mod", &profile).unwrap();

        assert!(profile.plugins_dir().join("Author-Mod/Loose.dll").is_file());
        assert!(profile.config_dir().join("mod.cfg").is_file());
        assert!(profile.patchers_dir().join("Author-Mod/Patch.dll").is_file());
        assert!(!stale.exists());
    }

    #[test]
    fn reads_manifest_with_bom() {
        let dir = tempfile::tempdir().unwrap();
        let zip = dir.path().join("Author-Mod-1.0.0.zip");
        let mut writer = s_zip::StreamingZipWriter::new(&zip).unwrap();
        writer.add_entry("manifest.json", "\u{feff}{\"name\":\"Mod\"}".as_bytes()).unwrap();
        writer.finish().unwrap();

        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        assert_eq!(rt.block_on(read_zip_manifest(&zip)).unwrap(), "{\"name\":\"Mod\"}");

        let empty = dir.path().join("broken.zip");
        s_zip::StreamingZipWriter::new(&empty).unwrap().finish().unwrap();
        assert!(rt.block_on(read_zip_manifest(&empty)).is_err());
    }

    #[test]
    fn hoist_skips_metadata() {
        let pack = tempfile::tempdir().unwrap();
        let profile_dir = tempfile::tempdir().unwrap();
        let root = pack.path();

        for file in ["manifest.json", "README.md", "icon.png", ".doorstop_version", "winhttp.dll"] {
            fs::write(root.join(file), "x").unwrap();
        }
        fs::create_dir_all(root.join("BepInEx/core")).unwrap();
        fs::write(root.join("BepInEx/core/BepInEx.Preloader.dll"), "x").unwrap();

        hoist_pack(root, profile_dir.path()).unwrap();

        let profile = Profile::new(profile_dir.path());
        assert!(profile_dir.path().join(".doorstop_version").is_file());
        assert!(profile_dir.path().join("winhttp.dll").is_file());
        assert!(profile.core_dir().join("BepInEx.Preloader.dll").is_file());
        assert!(!profile_dir.path().join("manifest.json").exists());
        assert!(!profile_dir.path().join("README.md").exists());
        assert!(!profile_dir.path().join("icon.png").exists());
    }
}
