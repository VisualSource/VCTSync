/// App folder inside the OS config dir; holds settings.ron and profiles.
const APP_DIR: &str = "vcl.visualsource.github";

/// Profile dirs may start with `%APPDATA%`, meaning `<config dir>/vcl.visualsource.github`
/// (see profile-list.tsx createProfile).
pub fn profile_from_dir(dir: &str) -> bepinex_lib::Profile {
    let path = match (dir.strip_prefix("%APPDATA%"), dirs::config_dir()) {
        (Some(rest), Some(config)) => config.join(APP_DIR).join(rest.trim_start_matches(['/', '\\'])),
        _ => std::path::PathBuf::from(dir),
    };
    bepinex_lib::Profile::new(normalize(&path))
}

/// Rebuild from components so separators are native (`/` becomes `\` on Windows).
/// Most APIs accept mixed separators but Explorer does not.
pub fn normalize(path: &std::path::Path) -> std::path::PathBuf {
    path.components().collect()
}

pub fn app_dir() -> std::io::Result<std::path::PathBuf> {
    dirs::config_dir()
        .map(|config| config.join(APP_DIR))
        .ok_or_else(|| std::io::Error::other("no config dir on this platform"))
}

/// Write `<app dir>/<name>` via a temp file + rename so a crash mid-save never leaves it truncated.
pub async fn write_app_file(name: &str, contents: String) -> std::io::Result<()> {
    let dir = app_dir()?;
    tokio::fs::create_dir_all(&dir).await?;
    let tmp = dir.join(format!("{name}.tmp"));
    tokio::fs::write(&tmp, contents).await?;
    tokio::fs::rename(&tmp, dir.join(name)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_paths_use_native_separators() {
        let root = profile_from_dir("%APPDATA%/profiles/abc").root().to_path_buf();
        assert!(root.ends_with("vcl.visualsource.github/profiles/abc"));
        if cfg!(windows) {
            assert!(!root.to_string_lossy().contains('/'), "{}", root.display());
            assert_eq!(normalize(std::path::Path::new("C:/Games/VC/log.txt")).to_string_lossy(), r"C:\Games\VC\log.txt");
        }
    }
}
