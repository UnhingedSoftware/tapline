//! Is a single install's throughput bounded by one connection pool?
//!
//! A single install tops out around 1.45 Gb/s on a 2.5 Gb link. Two pools in
//! this one process reach 1.83 Gb/s between them, which rules out the link, the
//! CPU and any per-client cap Steam might apply, and points the finger back at
//! us. This is the experiment that says so, kept because the conclusion is easy
//! to lose and expensive to rediscover.
//!
//! Note what it does *not* show: each `Shared` carries its own `HttpClient`, so
//! what differs between the two arms is the connection pool, not the CM
//! session. Chunk fetching never touches the session.
//!
//! ```sh
//! TAPLINE_LIVE_BIG=1 cargo test -p tapline --test saturate -- --ignored --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline::{AppId, InstallOptions, Session, Shared};

fn scratch(name: &str) -> std::path::PathBuf {
    let base = std::env::var("TAPLINE_TEST_DIR")
        .unwrap_or_else(|_| format!("{}/.cache/tapline-test", std::env::var("HOME").unwrap()));
    let path = std::path::PathBuf::from(base).join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("mkdir");
    path
}

const APP: AppId = AppId(896_660);
const WIRE: f64 = 1_467_655_792.0;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "downloads several GB from Steam"]
async fn two_connection_pools_beat_one() {
    if std::env::var("TAPLINE_LIVE_BIG").is_err() {
        println!("SKIPPED: set TAPLINE_LIVE_BIG=1");
        return;
    }

    // One session, for the baseline.
    let start = std::time::Instant::now();
    let mut session = Session::anonymous_shared(Shared::new(48))
        .await
        .expect("session");
    let options = InstallOptions {
        install_dir: scratch("sat-one"),
        concurrency: 48,
        ..InstallOptions::default()
    };
    session.install(APP, &options).await.expect("install");
    let one = start.elapsed().as_secs_f64();
    println!(
        "one session:  {one:.2}s  {:.2} Gb/s",
        WIRE * 8.0 / one / 1e9
    );

    // Two pools, same process, each its own Shared so neither is throttled by
    // the other's budget. Different directories, same app: if the cap were the
    // process, the link or Steam, this would not go faster. It does.
    let start = std::time::Instant::now();
    let a = tokio::spawn(async move {
        let mut s = Session::anonymous_shared(Shared::new(48)).await.expect("a");
        let o = InstallOptions {
            install_dir: scratch("sat-a"),
            concurrency: 48,
            ..InstallOptions::default()
        };
        s.install(APP, &o).await.expect("install a");
    });
    let b = tokio::spawn(async move {
        let mut s = Session::anonymous_shared(Shared::new(48)).await.expect("b");
        let o = InstallOptions {
            install_dir: scratch("sat-b"),
            concurrency: 48,
            ..InstallOptions::default()
        };
        s.install(APP, &o).await.expect("install b");
    });
    a.await.expect("join a");
    b.await.expect("join b");
    let two = start.elapsed().as_secs_f64();
    println!(
        "two sessions: {two:.2}s for 2x the bytes = {:.2} Gb/s aggregate",
        2.0 * WIRE * 8.0 / two / 1e9
    );

    let _ = std::fs::remove_dir_all(scratch("sat-one"));
    let _ = std::fs::remove_dir_all(scratch("sat-a"));
    let _ = std::fs::remove_dir_all(scratch("sat-b"));
}
