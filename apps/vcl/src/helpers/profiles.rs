use nanoserde::{DeJson, DeRon, SerRon};

use super::paths::{app_dir, write_app_file};
use crate::ipc_mappings::ProfileRecord;

/// Profiles live in `<app dir>/profiles.ron`. Falls back to the older profiles.json once;
/// the next save writes RON.
pub async fn load_profiles() -> std::io::Result<Vec<ProfileRecord>> {
    let dir = app_dir()?;
    match tokio::fs::read_to_string(dir.join("profiles.ron")).await {
        Ok(ron) => return Vec::<ProfileRecord>::deserialize_ron(&ron).map_err(std::io::Error::other),
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => return Err(err),
        Err(_) => {}
    }
    match tokio::fs::read_to_string(dir.join("profiles.json")).await {
        Ok(json) => Vec::<ProfileRecord>::deserialize_json(&json).map_err(std::io::Error::other),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err),
    }
}

/// `json` is the view's profile list.
pub async fn save_profiles(json: &str) -> std::io::Result<()> {
    let profiles = Vec::<ProfileRecord>::deserialize_json(json).map_err(std::io::Error::other)?;
    write_app_file("profiles.ron", profiles.serialize_ron()).await
}

/// Delete a profile folder, but only one inside the app folder: profile dirs are user-editable and
/// could point anywhere. Returns false (and leaves it) for anything outside.
pub fn delete_profile_dir(dir: &std::path::Path, app: &std::path::Path) -> std::io::Result<bool> {
    if !dir.exists() {
        return Ok(true);
    }
    if !app.exists() {
        return Ok(false);
    }
    let (dir, app) = (dir.canonicalize()?, app.canonicalize()?);
    if dir == app || !dir.starts_with(&app) {
        return Ok(false);
    }
    std::fs::remove_dir_all(dir)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nanoserde::{DeJson, DeRon, SerRon};

    #[test]
    fn profiles_drop_tanstack_keys_and_round_trip_ron() {
        // As saved by the view before the RON switch, TanStack's `$` row keys included.
        let json = r#"[{"id":"p1","name":"Profile","mods":[{"name":"BepInExPack","author":"BepInEx","version":"5.4.2305","icon":"","description":"pack","dependencies":[],"source":"thunderstore","id":"m1","enabled":true}],"updated_at":"2026-10-21T14:11:45","created_at":"2026-10-21T14:11:34","launch_args":"","dir":"%APPDATA%/profiles/p1","icon":"sword","$hasPendingWrites":true,"$synced":false,"$origin":"local","$key":"p1","$collectionId":"profiles"}]"#;

        let profiles = Vec::<ProfileRecord>::deserialize_json(json).unwrap();
        let ron = profiles.serialize_ron();
        assert!(!ron.contains('$'), "{ron}");

        let back = Vec::<ProfileRecord>::deserialize_ron(&ron).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].dir, "%APPDATA%/profiles/p1");
        assert_eq!(back[0].mods[0].version, "5.4.2305");
        assert!(back[0].mods[0].enabled);
    }

    #[test]
    fn delete_profile_dir_stays_inside_app_dir() {
        let root = std::env::temp_dir().join(format!("vcl-delete-test-{}", std::process::id()));
        let app = root.join("app");
        let inside = app.join("profiles/abc");
        let outside = root.join("elsewhere");
        std::fs::create_dir_all(inside.join("BepInEx")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();

        assert!(!delete_profile_dir(&outside, &app).unwrap());
        assert!(outside.is_dir());
        assert!(!delete_profile_dir(&app, &app).unwrap());
        assert!(!delete_profile_dir(&inside.join("../../.."), &app).unwrap());
        assert!(app.is_dir());

        assert!(delete_profile_dir(&inside, &app).unwrap());
        assert!(!inside.exists());
        assert!(delete_profile_dir(&inside, &app).unwrap(), "already gone is fine");

        std::fs::remove_dir_all(root).unwrap();
    }
}
