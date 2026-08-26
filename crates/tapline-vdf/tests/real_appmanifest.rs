//! The round-trip gate, against a file Valve's own tooling wrote.
//!
//! The unit tests in this crate use a fixture I typed, which proves the writer
//! agrees with my idea of Valve's layout. This one uses
//! `appmanifest_896660.acf` exactly as steamcmd produced it while installing
//! Valheim Dedicated Server on 2026-08-26 — so it proves the writer agrees with
//! Valve.
//!
//! The distinction matters more than it sounds. tapline shares install
//! directories with the real Steam client, steamcmd, LinuxGSM and every host
//! panel that has ever grepped an appmanifest. A file that came back
//! reformatted would still parse, and would still be a spurious diff for
//! everyone watching those files for changes.

use tapline_vdf::{Value, parse};

/// steamcmd's own output, byte for byte.
const REAL: &str = include_str!("fixtures/appmanifest_896660.acf");

#[test]
fn valves_own_appmanifest_round_trips_byte_for_byte() {
    let parsed = parse(REAL).expect("steamcmd's own file must parse");
    assert_eq!(
        parsed.to_string(),
        REAL,
        "rewriting steamcmd's appmanifest changed it"
    );
}

#[test]
fn the_fields_an_update_depends_on_are_readable() {
    let parsed = parse(REAL).expect("must parse");
    let state = parsed.get_object("AppState").expect("an AppState block");

    assert_eq!(state.get_u64("appid"), Some(896_660));
    assert_eq!(state.get_str("name"), Some("Valheim Dedicated Server"));
    assert_eq!(state.get_u64("buildid"), Some(21_981_590));
    assert_eq!(state.get_u64("SizeOnDisk"), Some(1_756_871_901));
    // The directory the content actually lives in, which is not the app's name.
    assert_eq!(
        state.get_str("installdir"),
        Some("Valheim dedicated server")
    );

    // The depot list is what a delta update diffs against: same depot, different
    // manifest means refetch only what changed.
    let depots = state
        .get_object("InstalledDepots")
        .expect("an InstalledDepots block");
    assert_eq!(depots.len(), 2);

    let depot = depots.get_object("896661").expect("the content depot");
    assert_eq!(depot.get_u64("manifest"), Some(962_159_520_942_340_660));
    assert_eq!(depot.get_u64("size"), Some(1_648_960_249));
}

#[test]
fn editing_one_field_leaves_every_other_byte_alone() {
    // An update rewrites the fields that changed and nothing else. Anything more
    // is a diff for everyone watching these files.
    let mut parsed = parse(REAL).expect("must parse");
    let mut state = parsed
        .get_object("AppState")
        .expect("an AppState block")
        .clone();

    state.set_str("buildid", "99999999");
    parsed.set("AppState", Value::Object(state));

    assert_eq!(
        parsed.to_string(),
        REAL.replace("21981590\"\n\t\"LastOwner", "99999999\"\n\t\"LastOwner"),
        "rewriting one field disturbed the rest of the file"
    );
}

#[test]
fn empty_blocks_survive() {
    // `UserConfig` and `MountedConfig` are empty in a fresh install and must not
    // disappear on rewrite — the Steam client expects them.
    let parsed = parse(REAL).expect("must parse");
    let state = parsed.get_object("AppState").expect("AppState");

    assert!(state.get_object("UserConfig").is_some_and(|o| o.is_empty()));
    assert!(
        state
            .get_object("MountedConfig")
            .is_some_and(|o| o.is_empty())
    );
    assert!(parsed.to_string().contains("\"MountedConfig\"\n\t{\n\t}"));
}
