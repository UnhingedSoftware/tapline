//! Garry's Mod Dedicated Server: the app that broke three assumptions.
//!
//! It is here as its own test rather than as another row in the differential
//! because each thing it caught was invisible to every app tested before it:
//!
//! - its depots mix **all three** chunk containers, and the container is a
//!   per-chunk property. A ZIP-wrapped chunk aborted the install outright until
//!   `tapline-chunk` learned the third one;
//! - it is 6.8 GB across 2,329 files in three depots, one of which (1006) is a
//!   shared redistributable depot rather than an app depot;
//! - installing it correctly is not the same as installing it **runnably**,
//!   which is what the mode assertion below is about.
//!
//! ```sh
//! TAPLINE_LIVE_BIG=1 cargo test -p tapline --test gmod -- --ignored --nocapture
//! ```
//!
//! 6.8 GB, so it is opt-in twice over: `--ignored` and the env var. It removes
//! its install tree afterwards, including on failure.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tapline::{AppId, FileModes, InstallOptions, Os, Session};

/// Garry's Mod Dedicated Server. Anonymous, and about 6.8 GB.
const APP: AppId = AppId(4020);

/// What a launcher runs. If these are not executable, the install is not a
/// server, whatever its bytes say.
const LAUNCHERS: &[&str] = &["srcds_run", "srcds_linux"];

/// Removes the install tree on the way out, panic or not — 6.8 GB is not
/// something to leave behind because an assertion fired.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch_root() -> PathBuf {
    let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
        |_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                .join(".cache/tapline-test")
        },
        PathBuf::from,
    );
    assert!(
        !base.starts_with("/tmp"),
        "the scratch root must not be tmpfs; a 6.8 GB install there is 6.8 GB of RAM"
    );
    base
}

/// Every regular file under `root`, with its mode.
fn walk(root: &Path) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            // The install record is ours, not depot content.
            if relative == "steamapps" || relative.starts_with("steamapps/") {
                continue;
            }
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                out.push((relative, metadata.permissions().mode() & 0o777));
            }
        }
    }
    out
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads 6.8 GB from Steam"]
async fn a_gmod_server_installs_ready_to_run() {
    if std::env::var("TAPLINE_LIVE_BIG").is_err() {
        println!("SKIPPED: set TAPLINE_LIVE_BIG=1 to download 6.8 GB");
        return;
    }

    let dir = scratch_root().join("gmod-ready");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    let scratch = Scratch(dir.clone());

    let options = InstallOptions {
        install_dir: dir.clone(),
        os: Os::Linux,
        branch: "public".to_owned(),
        ..InstallOptions::default()
    };

    let mut session = Session::anonymous().await.expect("anonymous session");
    let started = std::time::Instant::now();

    // The progress contract, asserted rather than eyeballed: a consumer drawing
    // a bar needs the denominator first and needs the numerator never to go
    // backwards, and both are easy to break without noticing.
    let mut events = Vec::new();
    let mut last_done = 0_u64;
    let mut monotonic = true;
    let mut progress_seen = 0_u64;
    let mut files_completed = 0_u64;
    let report = session
        .install_observed(APP, &options, &mut |event| {
            match &event {
                tapline::Event::Progress { bytes_done, .. } => {
                    if *bytes_done < last_done {
                        monotonic = false;
                    }
                    last_done = *bytes_done;
                    progress_seen += 1;
                }
                tapline::Event::FileCompleted { .. } => files_completed += 1,
                _ => {}
            }
            // Only the first few are kept: a 2,329-file install emits far too
            // many to hold, and the ordering claims below are about the head.
            if events.len() < 4 {
                events.push(event);
            }
        })
        .await
        .expect("install");

    assert!(
        matches!(events.first(), Some(tapline::Event::Planned { .. })),
        "the first event must be Planned, got {:?}",
        events.first()
    );
    assert!(monotonic, "Progress went backwards");
    assert!(progress_seen > 0, "no Progress events were emitted");
    assert_eq!(
        files_completed, report.files,
        "FileCompleted count must match the report"
    );
    println!(
        "installed {} files, {} bytes downloaded, in {:.1}s",
        report.files,
        report.bytes_downloaded,
        started.elapsed().as_secs_f64()
    );
    // Never silent about what was left out: a skipped file is how an install
    // ends up complete-looking and unrunnable.
    for (path, reason) in &report.skipped {
        println!("  skipped {path}: {reason}");
    }
    assert!(
        report.skipped.is_empty(),
        "{} files were skipped",
        report.skipped.len()
    );

    // All three depots, including the shared redistributable one.
    let mut depots: Vec<u32> = report.depots.iter().map(|depot| depot.get()).collect();
    depots.sort_unstable();
    assert_eq!(depots, vec![1006, 4021, 4023], "wrong depot set");

    // The launchers have to be runnable, whatever else is true.
    for name in LAUNCHERS {
        let path = dir.join(name);
        assert!(path.is_file(), "{name} is missing");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert!(mode & 0o111 != 0, "{name} is not executable ({mode:o})");
    }

    // steamclient.so is dlopened by the server at startup. Its absence is the
    // difference between a server and a process that exits.
    assert!(
        dir.join("steamclient.so").is_file(),
        "steamclient.so is missing; the server cannot initialise the Steam API"
    );

    // The default policy is steamcmd's: 0o755 on everything. Measured against a
    // steamcmd install of this same app, which sets exactly that on all 2,329
    // files.
    let files = walk(&dir);
    assert!(files.len() > 2_000, "only {} files installed", files.len());
    let odd: Vec<&(String, u32)> = files.iter().filter(|(_, mode)| *mode != 0o755).collect();
    for (path, mode) in odd.iter().take(20) {
        println!("  NOT 0o755: {path} is {mode:o}");
    }
    assert!(
        odd.is_empty(),
        "{} files are not 0o755 under the steamcmd mode policy",
        odd.len()
    );

    drop(scratch);
}

#[test]
fn the_manifest_policy_is_available_for_callers_that_want_it() {
    // The strict policy is a real option, not a documented intention.
    assert_eq!(FileModes::Manifest.mode_for(true), 0o755);
    assert_eq!(FileModes::Manifest.mode_for(false), 0o644);
    assert_eq!(FileModes::SteamCmd.mode_for(false), 0o755);
}
