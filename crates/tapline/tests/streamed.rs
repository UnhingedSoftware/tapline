#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::PathBuf;
use tapline::{Session, Window};
use tapline_ids::{AppId, PublishedFileId};

const ITEM: PublishedFileId = PublishedFileId(104_691_717);
#[allow(dead_code)]
const APP: AppId = AppId(4000);

fn scratch_root() -> PathBuf {
    let base = std::env::var("TAPLINE_TEST_DIR").unwrap_or_else(|_| {
        format!(
            "{}/.cache/tapline-test",
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        )
    });
    assert!(
        !base.starts_with("/tmp"),
        "the scratch root must not be tmpfs"
    );
    PathBuf::from(base)
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_addon_extracts_while_it_downloads() {
    let dest = scratch_root().join("streamed-addon");
    let _scratch = Scratch(dest.clone());
    let _ = std::fs::remove_dir_all(&dest);

    let mut session = Session::anonymous().await.expect("session");
    let details = session
        .workshop_details(&[ITEM])
        .await
        .expect("details")
        .into_iter()
        .next()
        .expect("one item")
        .expect("resolvable");

    let mut extractor = tapline_gmad::StreamingExtractor::new(&dest);
    let mut planned = 0_u64;

    let started = std::time::Instant::now();
    let report = session
        .stream_workshop_item(
            &details,
            Window::default(),
            &mut |bytes| {
                extractor
                    .push(bytes)
                    .map_err(|error| tapline::InstallError::Io(error.to_string()))
            },
            &mut |event| {
                if let tapline::Event::Planned { plan } = event {
                    planned = plan.total_bytes;
                }
            },
        )
        .await
        .expect("stream");

    let files = extractor.finish().expect("the stream must complete");
    let elapsed = started.elapsed();

    println!(
        "{} files in {:.2}s; {} chunks, {} bytes downloaded, peak {} chunks buffered",
        files.len(),
        elapsed.as_secs_f64(),
        report.chunks,
        report.bytes_downloaded,
        report.peak_buffered
    );

    assert_eq!(files.len(), 348, "PAC3 has 348 files");
    assert_eq!(report.bytes_streamed, planned, "streamed less than planned");

    assert!(
        !dest.join("104691717.gma").exists(),
        "the .gma was written after all"
    );

    assert!(
        report.peak_buffered <= Window::default().size,
        "buffered {} chunks, past the window of {}",
        report.peak_buffered,
        Window::default().size
    );

    let reference = scratch_root().join("streamed-reference");
    let _reference_scratch = Scratch(reference.clone());
    let _ = std::fs::remove_dir_all(&reference);
    std::fs::create_dir_all(&reference).expect("mkdir");

    let options = tapline::InstallOptions {
        install_dir: reference.clone(),
        workshop_layout: tapline::WorkshopLayout::Flat,
        ..tapline::InstallOptions::default()
    };
    session
        .download_workshop_item(&details, &options)
        .await
        .expect("reference download");

    let archive = reference.join("104691717.gma");
    let unpacked = reference.join("unpacked");
    let expected = tapline_gmad::extract(&archive, &unpacked).expect("reference extract");

    assert_eq!(
        files, expected,
        "the two paths produced different file lists"
    );
    for name in &files {
        assert_eq!(
            std::fs::read(dest.join(name)).expect("streamed file"),
            std::fs::read(unpacked.join(name)).expect("reference file"),
            "{name} differs between streaming and downloading"
        );
    }
    println!(
        "all {} files identical to the download-then-extract path",
        files.len()
    );
}
