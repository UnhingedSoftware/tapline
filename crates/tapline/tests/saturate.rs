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
