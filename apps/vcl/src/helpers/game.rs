use crate::{ipc_mappings::GameStatus, state::AppSettings};

/// Steam/game folders and whether the doorstop loader is in the game folder.
pub fn game_status(settings: &AppSettings) -> GameStatus {
    match settings.steam() {
        Ok(steam) => GameStatus {
            steam_dir: Some(steam.steam_dir.to_string_lossy().to_string()),
            game_dir: Some(steam.game_dir.to_string_lossy().to_string()),
            modded: steam.game_dir.join("winhttp.dll").is_file(),
            error: None,
        },
        Err(err) => GameStatus { steam_dir: None, game_dir: None, modded: false, error: Some(err.to_string()) },
    }
}
