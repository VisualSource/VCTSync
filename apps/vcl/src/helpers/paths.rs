/// App folder inside the OS config dir; holds settings.ron and profiles.
const APP_DIR: &str = "vcl.visualsource.github";

/// Profile dirs may start with `%APPDATA%`, meaning `<config dir>/vcl.visualsource.github`
/// (see profile-list.tsx createProfile).
pub fn profile_from_dir(dir: &str) -> bepinex_lib::Profile {
    match (dir.strip_prefix("%APPDATA%"), dirs::config_dir()) {
        (Some(rest), Some(config)) => {
            bepinex_lib::Profile::new(config.join(APP_DIR).join(rest.trim_start_matches(['/', '\\'])))
        }
        _ => bepinex_lib::Profile::new(dir),
    }
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
