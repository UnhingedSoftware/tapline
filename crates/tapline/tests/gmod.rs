#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tapline::{AppId, FileModes, InstallOptions, Os, Session};

const APP: AppId = AppId(4020);

const LAUNCHERS: &[&str] = &["srcds_run", "srcds_linux"];

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
    for (path, reason) in &report.skipped {
        println!("  skipped {path}: {reason}");
    }
    assert!(
        report.skipped.is_empty(),
        "{} files were skipped",
        report.skipped.len()
    );

    let mut depots: Vec<u32> = report.depots.iter().map(|depot| depot.get()).collect();
    depots.sort_unstable();
    assert_eq!(depots, vec![1006, 4021, 4023], "wrong depot set");

    for name in LAUNCHERS {
        let path = dir.join(name);
        assert!(path.is_file(), "{name} is missing");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert!(mode & 0o111 != 0, "{name} is not executable ({mode:o})");
    }

    assert!(
        dir.join("steamclient.so").is_file(),
        "steamclient.so is missing; the server cannot initialise the Steam API"
    );

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
    assert_eq!(FileModes::Manifest.mode_for(true), 0o755);
    assert_eq!(FileModes::Manifest.mode_for(false), 0o644);
    assert_eq!(FileModes::SteamCmd.mode_for(false), 0o755);
}
