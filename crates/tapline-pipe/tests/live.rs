//! The chain, against a real Workshop item.
//!
//! ```sh
//! cargo test -p tapline-pipe --test live -- --ignored --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::PathBuf;
use tapline::Session;

/// PAC3: 348 files, 8.4 MB, most of them under `lua/`.
const ITEM: u64 = 104_691_717;
const APP: u32 = 4000;

fn scratch(name: &str) -> PathBuf {
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
    let path = PathBuf::from(base).join(format!("pipe-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    path
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_chain_streams_into_a_zip_without_the_archive() {
    let root = scratch("tee");
    let _scratch = Scratch(root.clone());
    std::fs::create_dir_all(&root).expect("mkdir");

    let zip = root.join("out.zip");

    // No session: the pool provides one and takes it back.
    let outcome = tapline_pipe::workshop(APP, ITEM)
        .gma()
        .zip(zip.to_string_lossy())
        .run()
        .await
        .expect("the chain must run");

    println!(
        "{} entries, {} bytes streamed, peak {} chunks buffered",
        outcome.entries, outcome.bytes_streamed, outcome.peak_buffered
    );

    assert_eq!(outcome.entries, 348, "wrong entry count");

    assert!(zip.is_file(), "the zip was not written");

    // The archive itself never existed: that is the point of streaming.
    let stray: Vec<_> = std::fs::read_dir(&root)
        .expect("read root")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".gma"))
        .collect();
    assert!(stray.is_empty(), "a .gma was written: {stray:?}");

    if let Ok(output) = std::process::Command::new("unzip")
        .arg("-t")
        .arg(&zip)
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "unzip rejected it:\n{text}");
        println!("unzip -t accepted the zip");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_filter_selects_a_subset() {
    let root = scratch("filter");
    let _scratch = Scratch(root.clone());
    std::fs::create_dir_all(&root).expect("mkdir");

    // The manual path, on a session the caller owns.
    let mut session = Session::anonymous().await.expect("session");
    let outcome = tapline_pipe::workshop(APP, ITEM)
        .gma()
        .only("lua/**")
        .dir(root.to_string_lossy())
        .run_with(&mut session)
        .await
        .expect("run");

    println!(
        "{} of 348 entries matched lua/**: {} bytes fetched, {} written",
        outcome.entries, outcome.bytes_downloaded, outcome.bytes_streamed
    );
    assert!(outcome.entries > 0, "the filter matched nothing");
    assert!(
        outcome.entries < 348,
        "the filter matched everything, so it is not filtering"
    );

    assert!(root.join("lua").is_dir(), "lua/ was not written");
    assert!(
        !root.join("materials").exists(),
        "materials/ was written despite the filter"
    );

    // The point of reading by range: a filter now stops paying for what it
    // discards. The whole archive is 3.17 MB compressed; lua/** is a fraction.
    assert!(
        outcome.bytes_downloaded < 2_000_000,
        "a filtered run fetched {} bytes, which is most of the archive",
        outcome.bytes_downloaded
    );
    assert!(
        outcome.bytes_streamed < 8_000_000,
        "the filter wrote everything"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn concurrent_chains_do_not_wait_on_each_other() {
    // The reason the pool exists. Three chains at once, none of them holding a
    // session the others need, all sharing one chunk budget.
    let roots: Vec<PathBuf> = (0..3).map(|i| scratch(&format!("concurrent{i}"))).collect();
    let _guards: Vec<Scratch> = roots.iter().cloned().map(Scratch).collect();
    for root in &roots {
        std::fs::create_dir_all(root).expect("mkdir");
    }

    let started = std::time::Instant::now();
    let runs = roots.iter().map(|root| {
        tapline_pipe::workshop(APP, ITEM)
            .gma()
            .dir(root.to_string_lossy())
            .run()
    });
    let outcomes = futures_lite_join(runs).await;
    let elapsed = started.elapsed();

    for outcome in &outcomes {
        assert_eq!(
            outcome.as_ref().map(|o| o.entries).unwrap_or(0),
            348,
            "a concurrent chain did not finish"
        );
    }

    // And the pool kept the sessions rather than dropping them.
    let idle = tapline::SessionPool::shared().idle_count();
    println!(
        "three chains in {:.2}s, {idle} sessions kept",
        elapsed.as_secs_f64()
    );
    assert!(idle > 0, "no session was returned to the pool");
}

/// Joins futures without pulling in a futures crate for one test.
async fn futures_lite_join<F, T>(futures: impl Iterator<Item = F>) -> Vec<T>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handles: Vec<_> = futures.map(tokio::spawn).collect();
    let mut out = Vec::with_capacity(handles.len());
    for handle in handles {
        out.push(handle.await.expect("a chain task panicked"));
    }
    out
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_archive_can_be_listed_without_downloading_it() {
    // The question this answers: what is in this addon? Without paying for the
    // addon to find out.
    let listing = tapline_pipe::workshop(APP, ITEM)
        .gma()
        .list()
        .await
        .expect("list");

    println!(
        "{} entries known after reading {} of {} bytes",
        listing.entries.len(),
        listing.read_bytes,
        listing.archive_bytes
    );
    assert_eq!(listing.entries.len(), 348);
    assert!(
        listing.read_bytes < listing.archive_bytes / 10,
        "listing read {} of {} bytes, which is not much of a saving",
        listing.read_bytes,
        listing.archive_bytes
    );

    // Entries carry what a selective read needs.
    let first = listing.entries.first().expect("an entry");
    assert!(!first.path.is_empty());
    assert!(first.size > 0);

    // With no filter, everything is selected and costs the whole archive.
    assert_eq!(listing.selected.len(), listing.entries.len());
    assert_eq!(listing.selected_bytes, listing.total_bytes);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_listing_prices_a_filter_before_running_it() {
    let listing = tapline_pipe::workshop(APP, ITEM)
        .gma()
        .only("lua/**")
        .list()
        .await
        .expect("list");

    println!(
        "{} of {} entries selected: {} bytes to fetch against {}",
        listing.selected.len(),
        listing.entries.len(),
        listing.selected_bytes,
        listing.total_bytes
    );

    assert!(listing.selected.len() < listing.entries.len(), "no filtering");
    assert!(
        listing.selected_bytes < listing.total_bytes,
        "the filter does not reduce what would be fetched"
    );
    // Every selected entry actually matches.
    for entry in &listing.selected {
        assert!(
            entry.path.starts_with("lua/"),
            "{} was selected by lua/**",
            entry.path
        );
    }
}
