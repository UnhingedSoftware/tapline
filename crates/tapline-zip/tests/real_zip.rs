//! Against archives this crate did not write.
//!
//! The unit tests build ZIPs with the same understanding that reads them, which
//! proves self-consistency and nothing about the format. These are built by
//! Python's `zipfile`, an implementation with no connection to this one, so a
//! misreading shows up rather than cancelling out.
//!
//! ```sh
//! cargo test -p tapline-zip --test real_zip -- --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::PathBuf;
use tapline_ext::{Compression, IndexLocation};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(name: &str) -> Scratch {
    let base = std::env::var("TAPLINE_TEST_DIR").unwrap_or_else(|_| {
        format!(
            "{}/.cache/tapline-test",
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        )
    });
    let path = PathBuf::from(base).join(format!("zipread-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("mkdir");
    Scratch(path)
}

/// Builds an archive with Python, or `None` if Python is not here.
fn build(dir: &std::path::Path, script: &str) -> Option<Vec<u8>> {
    let archive = dir.join("made.zip");
    let program = format!("import zipfile, sys\nout = sys.argv[1]\n{script}\n",);
    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(&program)
        .arg(&archive)
        .output()
        .ok()?;
    if !output.status.success() {
        panic!(
            "python failed to build the archive: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::read(&archive).ok()
}

/// Reads an archive the way the pipeline does: tail, plan, headers, finalize.
fn read(raw: &[u8]) -> Vec<tapline_ext::ArchiveEntry> {
    let IndexLocation::Tail(want) = tapline_zip::index_location() else {
        panic!("a zip reads from the tail");
    };
    let start = raw.len().saturating_sub(want as usize);
    let plan = tapline_zip::plan(raw.get(start..).expect("tail"), start as u64).expect("plan");

    let headers: Vec<Vec<u8>> = plan
        .needs
        .iter()
        .map(|(offset, len)| {
            raw.get(*offset as usize..(*offset + *len) as usize)
                .expect("header in range")
                .to_vec()
        })
        .collect();
    tapline_zip::finalize(plan.entries, &headers).expect("finalize")
}

fn contents(raw: &[u8], entry: &tapline_ext::ArchiveEntry) -> Vec<u8> {
    let stored = raw
        .get(entry.offset as usize..(entry.offset + entry.stored_size) as usize)
        .unwrap_or_else(|| panic!("{} is out of range", entry.path));
    tapline_zip::decode(entry, stored).expect("decode")
}

#[test]
fn a_deflated_archive_from_another_implementation_reads_correctly() {
    let dir = scratch("deflated");
    let Some(raw) = build(
        &dir.0,
        r#"
z = zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED)
z.writestr("big.txt", "a" * 20000)
z.writestr("small.txt", "tiny")
z.writestr("nested/deep.txt", "down here")
z.close()
"#,
    ) else {
        println!("SKIPPED: no python3");
        return;
    };

    let entries = read(&raw);
    let names: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    println!("{} entries: {names:?}", entries.len());
    assert_eq!(names, vec!["big.txt", "small.txt", "nested/deep.txt"]);

    assert_eq!(contents(&raw, &entries[0]), vec![b'a'; 20_000]);
    assert_eq!(contents(&raw, &entries[1]), b"tiny");
    assert_eq!(contents(&raw, &entries[2]), b"down here");

    // The big one must actually have been deflated, or this proves nothing
    // about the inflate path.
    assert_eq!(entries[0].compression, Compression::Deflate);
    assert!(entries[0].stored_size < entries[0].size);
    println!("all three entries match, including a deflated one");
}

#[test]
fn a_stored_archive_reads_too() {
    let dir = scratch("stored");
    let Some(raw) = build(
        &dir.0,
        r#"
z = zipfile.ZipFile(out, "w", zipfile.ZIP_STORED)
z.writestr("a.bin", bytes([7]) * 5000)
z.close()
"#,
    ) else {
        println!("SKIPPED: no python3");
        return;
    };

    let entries = read(&raw);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].compression, Compression::Stored);
    assert_eq!(entries[0].stored_size, entries[0].size);
    assert_eq!(contents(&raw, &entries[0]), vec![7_u8; 5000]);
}

#[test]
fn an_end_record_behind_a_comment_is_still_found() {
    // A comment sits after the end record, and may contain anything — including
    // bytes that look like a signature. Scanning backwards finds the real one;
    // scanning forwards would find a decoy inside the data or the comment.
    let dir = scratch("comment");
    let Some(raw) = build(
        &dir.0,
        r#"
z = zipfile.ZipFile(out, "w", zipfile.ZIP_STORED)
z.writestr("a.txt", "body")
z.comment = b"a comment containing PK\x05\x06 as a decoy"
z.close()
"#,
    ) else {
        println!("SKIPPED: no python3");
        return;
    };

    let entries = read(&raw);
    assert_eq!(
        entries.len(),
        1,
        "the end record was not found past a comment"
    );
    assert_eq!(entries[0].path, "a.txt");
    assert_eq!(contents(&raw, &entries[0]), b"body");
}

#[test]
fn an_archive_with_extra_fields_lands_on_the_right_bytes() {
    // Python writes extra fields into local headers that the central directory
    // does not carry. A reader taking the data offset from the central record
    // alone would be several bytes early on every entry, and would produce
    // files that are wrong rather than missing.
    let dir = scratch("extra");
    let Some(raw) = build(
        &dir.0,
        r#"
import time
z = zipfile.ZipFile(out, "w", zipfile.ZIP_STORED)
info = zipfile.ZipInfo("a.txt", date_time=(2026, 8, 26, 12, 0, 0))
info.extra = b"\x99\x99\x08\x00" + b"12345678"
z.writestr(info, "exactly this")
z.close()
"#,
    ) else {
        println!("SKIPPED: no python3");
        return;
    };

    let entries = read(&raw);
    assert_eq!(entries.len(), 1);
    assert_eq!(contents(&raw, &entries[0]), b"exactly this");
    println!("data offset survived a local extra field");
}

#[test]
fn a_large_archive_keeps_its_directory_far_from_the_end() {
    // The case that needs a second read: with enough entries the central
    // directory begins before the tail window, so the reader must ask for it
    // rather than read past what it was given.
    let dir = scratch("large");
    let Some(raw) = build(
        &dir.0,
        r#"
z = zipfile.ZipFile(out, "w", zipfile.ZIP_STORED)
for i in range(3000):
    z.writestr("dir%04d/file%04d.txt" % (i, i), "x" * 32)
z.close()
"#,
    ) else {
        println!("SKIPPED: no python3");
        return;
    };

    let IndexLocation::Tail(want) = tapline_zip::index_location() else {
        panic!("tail");
    };
    let start = raw.len().saturating_sub(want as usize);
    let first = tapline_zip::plan(raw.get(start..).expect("tail"), start as u64).expect("plan");

    let entries = if first.entries.is_empty() && !first.needs.is_empty() {
        // The directory was outside the window; fetch exactly what was asked
        // for and read it.
        let (offset, len) = first.needs[0];
        println!("directory is {len} bytes at {offset}, outside a {want} byte tail");
        let directory = raw
            .get(offset as usize..(offset + len) as usize)
            .expect("directory in range");
        let plan = tapline_zip::read_directory(directory, 3000).expect("directory");
        let headers: Vec<Vec<u8>> = plan
            .needs
            .iter()
            .map(|(offset, len)| {
                raw.get(*offset as usize..(*offset + *len) as usize)
                    .expect("header")
                    .to_vec()
            })
            .collect();
        tapline_zip::finalize(plan.entries, &headers).expect("finalize")
    } else {
        read(&raw)
    };

    assert_eq!(entries.len(), 3000);
    assert_eq!(contents(&raw, &entries[2999]), vec![b'x'; 32]);
    println!(
        "read {} entries from a {} byte archive",
        entries.len(),
        raw.len()
    );
}
