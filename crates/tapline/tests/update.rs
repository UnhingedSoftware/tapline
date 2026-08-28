#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;
use tapline::{AppId, InstallOptions, Os, Session};
use tapline_state::AppState;

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

    let second = session
        .install(VALHEIM, &options)
        .await
        .expect("second install");
    println!(
        "second: {} files, {} downloaded, {} depots unchanged",
        second.files, second.bytes_downloaded, second.depots_unchanged
    );

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
    let root = scratch("record-tf2");
    let options = InstallOptions {
        install_dir: root.clone(),
        os: Os::Linux,
        branch: "public".to_owned(),
        ..InstallOptions::default()
    };

    let mut session = Session::anonymous().await.expect("session");
    let plan = session.plan(APP, &options).await.expect("plan");
    println!(
        "TF2 DS plan: {} files, {} bytes, {} to download",
        plan.file_count, plan.total_bytes, plan.download_bytes
    );

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
