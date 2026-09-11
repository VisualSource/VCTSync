mod error;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use crate::error::LibError;

const STREAM_VOID_CREW_APPID: &str = "1063420";
const BEPINEX_PRELOADER_PATH: &str = "BepInEx\\core\\BepInEx.Preloader.dll";

pub async fn install_bepin(
    version: &str,
    profile_dir: &Path,
    game_dir: &Path,
    http_client: &reqwest::Client,
) -> Result<(), LibError> {
    // 1. fetch version from github release

    let response = http_client.get("").send().await?;

    let stream = response.bytes_stream();

    // 2. extract to profile

    // 3. create "plugins" and "patches" folders
    let plugins_dir = profile_dir.join("BepInEx/plugins");

    if !plugins_dir.is_dir() {
        fs::create_dir_all(plugins_dir)?;
    }

    let patches_dir = profile_dir.join("BepInEx/patchers");
    if !patches_dir.is_dir() {
        fs::create_dir_all(patches_dir)?;
    }

    // 4. Copy "winhttp.dll" to game_dir
    let dll_to = game_dir.join("winhttp.dll");

    if !dll_to.is_file() {
        let dll_from = profile_dir.join("winhttp.dll");
        fs::copy(dll_from, dll_to)?;
    }

    // DONE
    todo!()
}

pub struct BepinExBootstrapper {
    stream_dir: PathBuf,
    game_dir: PathBuf,
    profile_dir: PathBuf,
}

impl BepinExBootstrapper {
    pub fn new() -> Self {
        unimplemented!()
    }

    fn get_bepin_preloader_dll_path(&self) -> Result<String, LibError> {
        let dll = self
            .profile_dir
            .join(BEPINEX_PRELOADER_PATH)
            .canonicalize()?;

        let result = dll.to_string_lossy().to_string();

        Ok(result)
    }

    pub fn get_doorstop_version(&self) -> Result<Option<String>, LibError> {
        let version_file = self.game_dir.join(".doorstop_version");

        if !version_file.is_file() {
            return Err(LibError::NoFileExists);
        }

        let data = fs::read_to_string(version_file)?;

        let line = data.trim().lines().next().map(|x| x.to_owned());

        Ok(line)
    }

    pub fn get_doorstop_config(&self) -> Result<(), LibError> {
        unimplemented!()
    }

    pub fn steam_launch(&self) -> Result<(), LibError> {
        let stream_exe = self.stream_dir.join("Steam.exe");

        let dll_path = self.get_bepin_preloader_dll_path()?;

        Command::new(stream_exe)
            .args([
                // Stream launch args
                "-applaunch",
                STREAM_VOID_CREW_APPID,
                // bepinEx args
                "--doorstop-enabled",
                "true",
                "--doorstop-target-assembly",
                &dll_path,
                // START game specific launch parameters
                "--mod-profile",
            ])
            .spawn()?;

        Ok(())
    }
}
