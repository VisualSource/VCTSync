use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LibError {
    #[error("steam install could not be found")]
    SteamNotFound,

    #[error("void crew is not installed in any steam library")]
    GameNotFound,

    #[error("no BepInEx preloader found in {0}")]
    PreloaderNotFound(PathBuf),

    #[error("doorstop is not installed in the game folder (missing {0})")]
    DoorstopNotInstalled(PathBuf),

    #[error("launching is only supported on windows")]
    UnsupportedPlatform,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("url parse: {0}")]
    UrlParse(String),

    #[error(transparent)]
    Zip(#[from] s_zip::SZipError),

    #[error(transparent)]
    Steam(#[from] steamlocate::Error),
}
