//! The differential gate: tapline's install must equal steamcmd's, file for
//! file and byte for byte.
//!
//! This is the assertion the whole project turns on. Everything else checks
//! that tapline is self-consistent; this checks that it agrees with the tool it
//! replaces.
//!
//! ```sh
//! # 1. Install with steamcmd first (see docs/PLAN.md).
//! # 2. Then:
//! TAPLINE_STEAMCMD_DIR=~/.cache/tapline-test/steamcmd-valheim \
//!   cargo test -p tapline --test differential -- --ignored --nocapture
//! ```
//!
//! Skipped rather than failed when the steamcmd tree is absent, because the
//! oracle is not something CI can produce — but the skip says so out loud, so a
//! green run never quietly means "the comparison did not happen".

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tapline::{AppId, InstallOptions, Os, Session};

/// Valheim Dedicated Server: anonymous, and about 1.7 GB.
const APP: AppId = AppId(896_660);

/// Files steamcmd creates that describe the install rather than being part of
/// it, and which tapline writes at M7 rather than M6.
const STEAMCMD_BOOKKEEPING: &[&str] = &["steamapps", ".steam"];

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
        "the scratch root must not be tmpfs; a 1.7 GB install there is 1.7 GB of RAM"
    );
    base
}

/// What the comparison knows about one installed file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    /// Size in bytes.
    size: u64,
    /// Permission bits.
    mode: u32,
}

/// Every regular file under `root`, by relative path.
fn walk(root: &Path) -> BTreeMap<String, Entry> {
    let mut out = BTreeMap::new();
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
            let relative_str = relative.to_string_lossy().replace('\\', "/");

            // steamcmd's own bookkeeping is not depot content.
            if STEAMCMD_BOOKKEEPING
                .iter()
                .any(|name| relative_str == *name || relative_str.starts_with(&format!("{name}/")))
            {
                continue;
            }

            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                use std::os::unix::fs::PermissionsExt;
                out.insert(
                    relative_str,
                    Entry {
                        size: metadata.len(),
                        mode: metadata.permissions().mode() & 0o777,
                    },
                );
            }
        }
    }
    out
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam and needs a steamcmd install to compare against"]
async fn tapline_installs_what_steamcmd_installs() {
    let Ok(reference) = std::env::var("TAPLINE_STEAMCMD_DIR") else {
        println!(
            "SKIPPED: set TAPLINE_STEAMCMD_DIR to a steamcmd install of app {APP} to run this"
        );
        return;
    };
    let reference = PathBuf::from(reference);
    assert!(
        reference.is_dir(),
        "TAPLINE_STEAMCMD_DIR does not exist: {}",
        reference.display()
    );

    let ours = scratch_root().join("tapline-valheim");
    let _ = std::fs::remove_dir_all(&ours);
    std::fs::create_dir_all(&ours).expect("scratch");

    println!("steamcmd: {}", reference.display());
    println!("tapline:  {}", ours.display());

    let options = InstallOptions {
        install_dir: ours.clone(),
        os: Os::Linux,
        branch: "public".to_owned(),
        include_dlc: false,
        ..InstallOptions::default()
    };

    let mut session = Session::anonymous().await.expect("anonymous session");
    println!("cell {}", session.cell_id());

    let plan = session.plan(APP, &options).await.expect("plan");
    println!(
        "plan: {} files, {} chunks, {} bytes on disk, {} to download",
        plan.file_count, plan.chunk_count, plan.total_bytes, plan.download_bytes
    );

    let started = std::time::Instant::now();
    let report = session.install(APP, &options).await.expect("install");
    let elapsed = started.elapsed();

    println!(
        "installed {} files, {} bytes written, {} downloaded, in {:.1}s",
        report.files,
        report.bytes_written,
        report.bytes_downloaded,
        elapsed.as_secs_f64()
    );
    for (path, reason) in &report.skipped {
        println!("  skipped {path}: {reason}");
    }

    // --- the comparison ---------------------------------------------------
    let theirs = walk(&reference);
    let mine = walk(&ours);
    println!(
        "steamcmd: {} files, tapline: {} files",
        theirs.len(),
        mine.len()
    );

    let missing: Vec<&String> = theirs
        .keys()
        .filter(|path| !mine.contains_key(*path))
        .collect();
    let extra: Vec<&String> = mine
        .keys()
        .filter(|path| !theirs.contains_key(*path))
        .collect();

    for path in missing.iter().take(20) {
        println!("  MISSING from tapline: {path}");
    }
    for path in extra.iter().take(20) {
        println!("  EXTRA in tapline: {path}");
    }

    assert!(
        missing.is_empty(),
        "{} files steamcmd installed are absent",
        missing.len()
    );
    assert!(
        extra.is_empty(),
        "{} files tapline installed are not in steamcmd's tree",
        extra.len()
    );

    // Sizes, then contents. Size first because a mismatch there localises the
    // problem faster than a hash over a gigabyte does.
    let mut wrong_size = Vec::new();
    for (path, entry) in &theirs {
        if mine.get(path).map(|mine| mine.size) != Some(entry.size) {
            wrong_size.push(path.clone());
        }
    }
    for path in wrong_size.iter().take(20) {
        println!("  SIZE MISMATCH: {path}");
    }
    assert!(
        wrong_size.is_empty(),
        "{} files differ in size",
        wrong_size.len()
    );

    // Modes, which this test did not compare until 2026-08-26 — and so did not
    // notice that steamcmd sets 0o755 on every file it writes while tapline was
    // applying the manifest's executable flag. 2,291 of Garry's Mod's 2,329
    // files disagreed, with byte-identical contents throughout. A comparison
    // that skips an attribute silently reports parity it never checked.
    let mut wrong_mode = Vec::new();
    for (path, entry) in &theirs {
        if mine.get(path).map(|mine| mine.mode) != Some(entry.mode) {
            wrong_mode.push(path.clone());
        }
    }
    for path in wrong_mode.iter().take(20) {
        println!(
            "  MODE MISMATCH: {path}  steamcmd={:o} tapline={:o}",
            theirs[path].mode,
            mine.get(path).map_or(0, |entry| entry.mode)
        );
    }
    assert!(
        wrong_mode.is_empty(),
        "{} files differ in permissions",
        wrong_mode.len()
    );

    let mut wrong_content = Vec::new();
    for path in theirs.keys() {
        let a = std::fs::read(reference.join(path)).expect("read steamcmd's copy");
        let b = std::fs::read(ours.join(path)).expect("read tapline's copy");
        if a != b {
            wrong_content.push(path.clone());
        }
    }
    for path in wrong_content.iter().take(20) {
        println!("  CONTENT MISMATCH: {path}");
    }
    assert!(
        wrong_content.is_empty(),
        "{} files differ in content",
        wrong_content.len()
    );

    println!(
        "IDENTICAL: {} files, byte for byte, against steamcmd",
        theirs.len()
    );

    // The install is large; leave nothing behind.
    let _ = std::fs::remove_dir_all(&ours);
}
