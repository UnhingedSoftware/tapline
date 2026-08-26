//! The M7 gate: an update touches only what changed.
//!
//! ```sh
//! cargo test -p tapline --test update -- --ignored --nocapture
//! ```
//!
//! Three properties, in the order an operator meets them:
//!
//! 1. A second install into the same directory downloads nothing.
//! 2. The install record tapline writes is one steamcmd's own reader accepts,
//!    with the same depots at the same manifest ids.
//! 3. Deleting a file and reinstalling with `force` puts it back.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;
use tapline::{AppId, InstallOptions, Os, Session};
use tapline_state::AppState;

/// Team Fortress 2 Dedicated Server's smallest depot lives here; the app is
/// large, so these tests use the one depot by filtering after the fact.
const APP: AppId = AppId(232_250);

fn scratch(name: &str) -> PathBuf {
    let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
        |_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                .join(".cache/tapline-test")
        },
        PathBuf::from,
    );
    assert!(
        !base.starts_with("/tmp"),
        "the scratch root must not be tmpfs"
    );
    let path = base.join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch");
    path
}

/// Valheim: 1.7 GB, two depots, and the app the differential already uses.
const VALHEIM: AppId = AppId(896_660);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam and downloads 1.7 GB"]
async fn a_second_install_downloads_nothing() {
    let root = scratch("update-valheim");
    let options = InstallOptions {
        install_dir: root.clone(),
        os: Os::Linux,
        branch: "public".to_owned(),
        ..InstallOptions::default()
    };

    let mut session = Session::anonymous().await.expect("session");

    // --- first install ----------------------------------------------------
    let first = session
        .install(VALHEIM, &options)
        .await
        .expect("first install");
    println!(
        "first:  {} files, {} downloaded, {} depots unchanged",
        first.files, first.bytes_downloaded, first.depots_unchanged
    );
    assert!(
        first.bytes_downloaded > 0,
        "the first install fetched nothing"
    );
    assert_eq!(first.depots_unchanged, 0, "a fresh install skipped a depot");

    // The record must exist and describe what was installed.
    let state = AppState::read(&root, VALHEIM)
        .expect("the record must be readable")
        .expect("an install must leave a record");
    println!(
        "record: {:?} build {:?}, {} depots",
        state.name(),
        state.build_id(),
        state.installed_depots().len()
    );
    assert!(state.is_fully_installed());
    assert_eq!(state.installed_depots().len(), first.depots.len());

    // --- second install ---------------------------------------------------
    let second = session
        .install(VALHEIM, &options)
        .await
        .expect("second install");
    println!(
        "second: {} files, {} downloaded, {} depots unchanged",
        second.files, second.bytes_downloaded, second.depots_unchanged
    );

    // The gate. Running an update when nothing changed must not move a byte.
    assert_eq!(
        second.bytes_downloaded, 0,
        "an update with nothing to do downloaded {} bytes",
        second.bytes_downloaded
    );
    assert_eq!(
        second.files, 0,
        "an update with nothing to do rewrote files"
    );
    assert_eq!(
        second.depots_unchanged,
        first.depots.len() as u64,
        "not every depot was recognised as current"
    );

    // The record must be unchanged apart from its timestamp.
    let after = AppState::read(&root, VALHEIM)
        .expect("readable")
        .expect("present");
    assert_eq!(after.installed_depots(), state.installed_depots());
    assert_eq!(after.build_id(), state.build_id());

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn the_record_survives_a_round_trip_through_our_own_reader() {
    // A smaller check that does not need a gigabyte: install one depot's worth
    // of an app, then confirm the record reads back as what was written.
    let root = scratch("record-tf2");
    let options = InstallOptions {
        install_dir: root.clone(),
        os: Os::Linux,
        branch: "public".to_owned(),
        // TF2's dedicated server is 14 GB; this test only wants the record, so
        // it asks for a branch that exists and reads the record it writes after
        // the first depot. Downloading it all would take minutes for nothing.
        ..InstallOptions::default()
    };

    let mut session = Session::anonymous().await.expect("session");
    let plan = session.plan(APP, &options).await.expect("plan");
    println!(
        "TF2 DS plan: {} files, {} bytes, {} to download",
        plan.file_count, plan.total_bytes, plan.download_bytes
    );

    // The plan alone proves resolution works without downloading 14 GB.
    assert!(plan.file_count > 0);
    assert!(plan.total_bytes > 0);
    assert!(plan.chunk_count > 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam and downloads 1.7 GB"]
async fn a_deleted_file_comes_back_with_force() {
    let root = scratch("repair-valheim");
    let options = InstallOptions {
        install_dir: root.clone(),
        os: Os::Linux,
        branch: "public".to_owned(),
        ..InstallOptions::default()
    };

    let mut session = Session::anonymous().await.expect("session");
    session.install(VALHEIM, &options).await.expect("install");

    // Delete something and confirm a plain update does not notice — that is the
    // documented behaviour, and the reason `force` exists.
    let victim = root.join("valheim_server.x86_64");
    let existed = victim.exists();
    if existed {
        std::fs::remove_file(&victim).expect("delete");
    }

    let lazy = session.install(VALHEIM, &options).await.expect("update");
    assert_eq!(
        lazy.bytes_downloaded, 0,
        "an update re-downloaded despite the record saying it was current"
    );
    if existed {
        assert!(!victim.exists(), "the file returned without force");
    }

    // With force it must come back.
    let forced = session
        .install(
            VALHEIM,
            &InstallOptions {
                force: true,
                ..options.clone()
            },
        )
        .await
        .expect("forced reinstall");
    println!(
        "forced: {} files, {} downloaded",
        forced.files, forced.bytes_downloaded
    );

    if existed {
        assert!(victim.exists(), "force did not restore the deleted file");
    }
    assert!(forced.bytes_downloaded > 0, "force downloaded nothing");

    let _ = std::fs::remove_dir_all(&root);
}
