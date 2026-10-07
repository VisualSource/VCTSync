use std::{
    ffi::OsString,
    process::{Child, Command},
};

use crate::{
    VOID_CREW_APP_ID,
    doorstop::{DOORSTOP_PROXY_DLL, DoorstopVersion},
    error::LibError,
    profile::Profile,
    steam::SteamPaths,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// Load BepInEx from the profile.
    Modded,
    /// Explicitly disable doorstop.
    Vanilla,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMethod {
    /// `Steam.exe -applaunch` (overlay, playtime, steam auth).
    Steam,
    /// Run `Void Crew.exe` directly; Steam must already be running.
    Direct,
}

#[derive(Debug, Clone)]
pub struct Launcher {
    pub steam: SteamPaths,
    pub profile: Profile,
}

impl Launcher {
    pub fn new(steam: SteamPaths, profile: Profile) -> Self {
        Self { steam, profile }
    }

    /// Doorstop version installed in the game folder.
    pub fn doorstop_version(&self) -> DoorstopVersion {
        DoorstopVersion::detect(self.steam.game_dir())
    }

    /// Build the launch command without running it.
    pub fn build_command<I, S>(
        &self,
        mode: LaunchMode,
        method: LaunchMethod,
        extra_args: I,
    ) -> Result<Command, LibError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let version = self.doorstop_version();

        let doorstop_args = match mode {
            LaunchMode::Modded => version.modded_args(&self.profile.find_preloader()?),
            LaunchMode::Vanilla => version.vanilla_args(),
        };

        let mut command = match method {
            LaunchMethod::Steam => {
                let mut command = Command::new(self.steam.steam_exe());
                command.arg("-applaunch").arg(VOID_CREW_APP_ID.to_string());
                command
            }
            LaunchMethod::Direct => {
                let mut command = Command::new(self.steam.game_exe());
                command.current_dir(self.steam.game_dir());
                command
            }
        };

        command
            .args(doorstop_args)
            .args(extra_args.into_iter().map(Into::into));

        Ok(command)
    }

    /// Launch the game. Modded launches require the profile to be linked
    /// into the game folder first (see [`crate::link_profile`]).
    pub fn launch<I, S>(
        &self,
        mode: LaunchMode,
        method: LaunchMethod,
        extra_args: I,
    ) -> Result<Child, LibError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        if mode == LaunchMode::Modded {
            let proxy = self.steam.game_dir().join(DOORSTOP_PROXY_DLL);
            if !proxy.is_file() {
                return Err(LibError::DoorstopNotInstalled(proxy));
            }
        }

        let mut command = self.build_command(mode, method, extra_args)?;
        spawn(&mut command)
    }
}

#[cfg(windows)]
fn spawn(command: &mut Command) -> Result<Child, LibError> {
    Ok(command.spawn()?)
}

#[cfg(not(windows))]
fn spawn(_command: &mut Command) -> Result<Child, LibError> {
    Err(LibError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, fs, path::Path};

    use super::*;

    fn setup(game: &Path, profile: &Path) -> Launcher {
        let profile = Profile::new(profile);
        fs::create_dir_all(profile.core_dir()).unwrap();
        fs::write(profile.core_dir().join("BepInEx.Preloader.dll"), "").unwrap();
        fs::write(game.join(".doorstop_version"), "4.5.0").unwrap();

        Launcher::new(SteamPaths::new("/steam", game), profile)
    }

    fn args(command: &Command) -> Vec<&OsStr> {
        command.get_args().collect()
    }

    #[test]
    fn steam_modded() {
        let game = tempfile::tempdir().unwrap();
        let profile = tempfile::tempdir().unwrap();
        let launcher = setup(game.path(), profile.path());

        let command = launcher
            .build_command(LaunchMode::Modded, LaunchMethod::Steam, ["-extra"])
            .unwrap();
        let preloader = launcher.profile.find_preloader().unwrap();

        assert_eq!(command.get_program(), Path::new("/steam").join("Steam.exe"));
        assert_eq!(
            args(&command),
            [
                OsStr::new("-applaunch"),
                OsStr::new("1063420"),
                OsStr::new("--doorstop-enabled"),
                OsStr::new("true"),
                OsStr::new("--doorstop-target-assembly"),
                preloader.as_os_str(),
                OsStr::new("-extra"),
            ]
        );
        assert_eq!(command.get_current_dir(), None);
    }

    #[test]
    fn direct_vanilla() {
        let game = tempfile::tempdir().unwrap();
        let profile = tempfile::tempdir().unwrap();
        let launcher = setup(game.path(), profile.path());

        let command = launcher
            .build_command(LaunchMode::Vanilla, LaunchMethod::Direct, None::<OsString>)
            .unwrap();

        assert_eq!(command.get_program(), game.path().join("Void Crew.exe"));
        assert_eq!(args(&command), ["--doorstop-enabled", "false"]);
        assert_eq!(command.get_current_dir(), Some(game.path()));
    }

    #[test]
    fn modded_requires_proxy_dll() {
        let game = tempfile::tempdir().unwrap();
        let profile = tempfile::tempdir().unwrap();
        let launcher = setup(game.path(), profile.path());

        assert!(matches!(
            launcher.launch(LaunchMode::Modded, LaunchMethod::Steam, None::<OsString>),
            Err(LibError::DoorstopNotInstalled(_))
        ));
    }
}
