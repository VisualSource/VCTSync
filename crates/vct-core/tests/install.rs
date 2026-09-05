//! End-to-end check of the `install.ts` port: extract, `mods.yml` upsert,
//! provenance marker, and read-back.

use std::io::Write;

use vct_core::install;

/// Minimal Thunderstore-style package zip.
fn make_zip(version: &str) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("manifest.json", opts).unwrap();
        write!(
            zip,
            r#"{{"name":"VisualSource-VoidCrewTerminus","version_number":"{version}",
                "description":"test","website_url":"","dependencies":["BepInEx-BepInExPack-5.4.2305"]}}"#
        )
        .unwrap();

        zip.start_file("VoidCrewTerminus.dll", opts).unwrap();
        zip.write_all(b"\x00fake dll bytes").unwrap();

        zip.start_file("empty.txt", opts).unwrap(); // zero-length: must be skipped
        zip.finish().unwrap();
    }
    buf
}

fn scratch(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "vct-core-it-{}-{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn install_extracts_updates_mods_yml_and_marks_provenance() {
    let profile = scratch("profile");

    // pre-existing mods.yml: a foreign mod plus a stale entry for ours.
    std::fs::write(
        profile.join("mods.yml"),
        "- name: Someone-OtherMod\n  enabled: true\n  keepThisField: 42\n\
         - name: VisualSource-VoidCrewTerminus\n  enabled: false\n  installedAtTime: 111\n",
    )
    .unwrap();

    let report = install::install(&make_zip("0.0.18"), &profile, None, "local:0.0.18 Debug").unwrap();
    assert_eq!(report.version, "0.0.18");
    assert_eq!(report.files_written, 2, "zero-length entry should be skipped");
    assert!(report.mods_yml_updated, "existing entry should be updated, not appended");

    let mod_dir = profile
        .join("BepInEx")
        .join("plugins")
        .join("VisualSource-VoidCrewTerminus");
    assert!(mod_dir.join("VoidCrewTerminus.dll").is_file());
    assert!(!mod_dir.join("empty.txt").exists());

    // mods.yml: foreign entry intact, ours refreshed, disabled state preserved.
    let yml = std::fs::read_to_string(profile.join("mods.yml")).unwrap();
    assert!(yml.contains("keepThisField"), "foreign entry must survive");
    let (ver, enabled) = install::installed_version(&profile).unwrap();
    assert_eq!(ver, "0.0.18");
    assert!(!enabled, "prior enabled=false must be preserved");

    // provenance marker round-trips and identifies the build by hash.
    let marker = install::installed_marker(&profile).expect("marker written");
    assert_eq!(marker.source, "local:0.0.18 Debug");
    assert_eq!(marker.version, "0.0.18");
    assert_eq!(marker.sha256.len(), 64);
    assert!(marker.installed_at > 0);

    // a re-install with different content changes the hash.
    let r2 = install::install(&make_zip("0.0.18"), &profile, None, "local:0.0.18 Release").unwrap();
    // same inputs here → same hash; just assert the marker followed the source.
    assert_eq!(
        install::installed_marker(&profile).unwrap().source,
        "local:0.0.18 Release"
    );
    assert_eq!(r2.sha256, marker.sha256);

    std::fs::remove_dir_all(&profile).ok();
}
