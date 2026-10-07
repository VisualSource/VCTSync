use std::{fs, path::Path};

use reqwest::{Response, Url};
use tokio::io::AsyncWriteExt;

use crate::{error::LibError, profile::Profile};

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
