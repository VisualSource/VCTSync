use std::{path::Path, time::UNIX_EPOCH};

use crate::ipc_mappings::LogFile;

/// Read a profile's `LogOutput.log`. `content` is left out when the file's size and modified time
/// still match `known_size`/`known_modified` (what the view already has), so polling is cheap.
/// A missing file is not an error: the profile just hasn't been launched yet.
pub fn read_log(file: &Path, known_size: u64, known_modified: u64) -> std::io::Result<LogFile> {
    let path = file.to_string_lossy().to_string();
    let meta = match std::fs::metadata(file) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LogFile { path, exists: false, size: 0, modified: 0, content: None });
        }
        Err(err) => return Err(err),
    };

    let size = meta.len();
    let modified = meta
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();

    // BepInEx keeps the log open while the game runs, but shares it for reading.
    let content = (size != known_size || modified != known_modified)
        .then(|| std::fs::read(file).map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
        .transpose()?;

    Ok(LogFile { path, exists: true, size, modified, content })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_log_skips_unchanged_and_handles_missing() {
        let dir = std::env::temp_dir().join(format!("vcl-log-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("LogOutput.log");

        let missing = read_log(&file, 0, 0).unwrap();
        assert!(!missing.exists && missing.content.is_none());

        std::fs::write(&file, "[Info   :   BepInEx] hi\n").unwrap();
        let first = read_log(&file, 0, 0).unwrap();
        assert_eq!(first.content.as_deref(), Some("[Info   :   BepInEx] hi\n"));

        let again = read_log(&file, first.size, first.modified).unwrap();
        assert!(again.exists && again.content.is_none(), "unchanged file sends no content");

        std::fs::remove_dir_all(dir).unwrap();
    }
}
