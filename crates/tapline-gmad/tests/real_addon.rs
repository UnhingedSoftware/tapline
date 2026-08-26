//! Against a real addon, not a constructed one.
//!
//! The unit tests build archives with the same code that reads them, which
//! proves self-consistency and nothing about Valve's format. This reads PAC3
//! (Workshop item 104691717) as Steam serves it.
//!
//! ```sh
//! tapline workshop download 4000 104691717 --dir ~/.cache/tapline-test/gma --flat
//! cargo test -p tapline-gmad --test real_addon -- --ignored --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::PathBuf;

fn addon_path() -> PathBuf {
    let base = std::env::var("TAPLINE_TEST_DIR").unwrap_or_else(|_| {
        format!(
            "{}/.cache/tapline-test",
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        )
    });
    PathBuf::from(base).join("gma/104691717.gma")
}

/// Removes its directory on the way out, including on panic.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "needs a real addon downloaded first"]
fn a_real_addon_parses_and_extracts() {
    let path = addon_path();
    if !path.is_file() {
        println!("SKIPPED: no addon at {}", path.display());
        return;
    }

    let (addon, content_offset) = tapline_gmad::read_index(&path).expect("must read the index");
    println!(
        "{:?} v{} by {:?}, {} files, {} bytes unpacked, contents at {content_offset}",
        addon.name,
        addon.version,
        addon.author,
        addon.entries.len(),
        addon.unpacked_size()
    );

    assert_eq!(addon.name, "PAC3");
    assert_eq!(addon.version, 3);
    assert!(!addon.entries.is_empty(), "an addon with no files");

    // The index must describe the file exactly: every entry inside the archive,
    // and the last one ending at or before its end.
    let on_disk = std::fs::metadata(&path).expect("stat").len();
    for entry in &addon.entries {
        let end = entry.offset as u64 + entry.size;
        assert!(
            end <= on_disk,
            "{:?} runs to {end}, past the archive's {on_disk}",
            entry.path
        );
    }

    let dest = path.with_file_name("extracted-test");
    let _scratch = Scratch(dest.clone());
    let _ = std::fs::remove_dir_all(&dest);

    let written = tapline_gmad::extract(&path, &dest).expect("must extract");
    assert_eq!(written.len(), addon.entries.len());

    // Every file must exist at the size the index promised.
    for entry in &addon.entries {
        let target = dest.join(&entry.path);
        let meta = std::fs::metadata(&target)
            .unwrap_or_else(|error| panic!("{:?} was not written: {error}", entry.path));
        assert_eq!(meta.len(), entry.size, "{:?} is the wrong size", entry.path);
    }

    // And the bytes must be right, checked against the archive's own CRC where
    // it bothered to record one.
    let raw = std::fs::read(&path).expect("read");
    let mut checked = 0;
    for entry in &addon.entries {
        if let Some(ok) = addon.verify(entry, &raw) {
            assert!(ok, "{:?} failed its CRC inside the archive", entry.path);
            let written = std::fs::read(dest.join(&entry.path)).expect("read written");
            assert_eq!(
                crc32fast::hash(&written),
                entry.crc,
                "{:?} was written with different bytes than the archive holds",
                entry.path
            );
            checked += 1;
        }
    }
    println!("{checked} of {} entries carried a CRC", addon.entries.len());
}

#[test]
#[ignore = "needs a real addon downloaded first"]
fn a_real_addon_converts_to_a_zip_other_tools_can_open() {
    let path = addon_path();
    if !path.is_file() {
        println!("SKIPPED: no addon at {}", path.display());
        return;
    }

    let zip = path.with_file_name("converted-test.zip");
    let _ = std::fs::remove_file(&zip);
    let count = tapline_gmad::to_zip(&path, &zip, true).expect("must convert");

    let (addon, _) = tapline_gmad::read_index(&path).expect("index");
    assert_eq!(count, addon.entries.len());

    let size = std::fs::metadata(&zip).expect("stat").len();
    println!(
        "{} entries, {} bytes unpacked -> {size} bytes zipped",
        count,
        addon.unpacked_size()
    );

    // The claim worth testing is not that we can read our own output. It is
    // that something else can: a ZIP that only this crate accepts is not a ZIP.
    let unzip = std::process::Command::new("unzip")
        .arg("-t")
        .arg(&zip)
        .output();
    match unzip {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            assert!(
                output.status.success(),
                "unzip -t rejected the archive:\n{text}{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                text.contains("No errors detected"),
                "unzip did not confirm the archive:\n{text}"
            );
            println!("unzip -t accepted it");
        }
        Err(error) => println!("SKIPPED the unzip check: {error}"),
    }

    let _ = std::fs::remove_file(&zip);
}

#[test]
#[ignore = "a benchmark, not a gate"]
fn how_long_the_work_actually_takes() {
    let path = addon_path();
    if !path.is_file() {
        println!("SKIPPED: no addon at {}", path.display());
        return;
    }
    let (addon, _) = tapline_gmad::read_index(&path).expect("index");
    let bytes = addon.unpacked_size();

    let time = |label: &str, mut work: Box<dyn FnMut()>| {
        // Three passes: the first pays for a cold page cache, and reporting
        // that as the cost of the code would be measuring the disk.
        let mut best = std::time::Duration::MAX;
        for _ in 0..3 {
            let start = std::time::Instant::now();
            work();
            best = best.min(start.elapsed());
        }
        let mb = bytes as f64 / 1_048_576.0;
        println!(
            "{label:20} {:>7.1} ms  {:>7.0} MB/s",
            best.as_secs_f64() * 1000.0,
            mb / best.as_secs_f64()
        );
        best
    };

    let dest = path.with_file_name("bench-extract");
    let _scratch = Scratch(dest.clone());
    time(
        "extract",
        Box::new(|| {
            let _ = std::fs::remove_dir_all(&dest);
            tapline_gmad::extract(&path, &dest).expect("extract");
        }),
    );

    let zip = path.with_file_name("bench.zip");
    time(
        "to_zip (deflate)",
        Box::new(|| {
            tapline_gmad::to_zip(&path, &zip, true).expect("zip");
        }),
    );
    let deflated = std::fs::metadata(&zip).expect("stat").len();

    time(
        "to_zip (stored)",
        Box::new(|| {
            tapline_gmad::to_zip(&path, &zip, false).expect("zip");
        }),
    );
    let stored = std::fs::metadata(&zip).expect("stat").len();
    let _ = std::fs::remove_file(&zip);

    println!(
        "{} files, {bytes} bytes unpacked; zip deflated {deflated}, stored {stored}",
        addon.entries.len()
    );
}
