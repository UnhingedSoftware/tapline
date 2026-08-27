//! Searching a real Workshop.
//!
//! ```sh
//! cargo test --release -p tapline --test browse -- --ignored --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline::{AppId, BrowseQuery, BrowseSort, Session};

/// Garry's Mod, whose Workshop is large enough that paging is real.
const APP: AppId = AppId(4000);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_anonymous_session_can_search() {
    let mut session = Session::anonymous().await.expect("session");
    let page = session
        .browse_workshop(&BrowseQuery {
            app: APP,
            per_page: 5,
            ..BrowseQuery::default()
        })
        .await
        .expect("search");

    println!(
        "{} of {} items, {} skipped, more: {}",
        page.items.len(),
        page.total,
        page.skipped.len(),
        page.has_more()
    );
    for found in &page.items {
        println!(
            "  {} {:?} {} bytes, {} subs, tags {:?}",
            found.item.id, found.item.title, found.item.size, found.subscriptions, found.tags
        );
    }

    assert!(!page.items.is_empty(), "a search returned nothing");
    assert!(
        page.total > 1000,
        "GMod should have more than {} items",
        page.total
    );
    assert!(
        page.items.iter().all(|found| !found.item.title.is_empty()),
        "every result should have a title"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn the_cursor_walks_forward() {
    // The property that makes paging usable: a second page is different items,
    // not the same ones again. An offset-based pager silently repeats here.
    let mut session = Session::anonymous().await.expect("session");
    let query = BrowseQuery {
        app: APP,
        per_page: 10,
        ..BrowseQuery::default()
    };

    let first = session.browse_workshop(&query).await.expect("page one");
    assert!(first.has_more(), "the first page should not be the last");

    let second = session
        .browse_workshop(&BrowseQuery {
            cursor: first.next_cursor.clone(),
            ..query
        })
        .await
        .expect("page two");

    let ids: Vec<_> = first.items.iter().map(|f| f.item.id.get()).collect();
    let overlap = second
        .items
        .iter()
        .filter(|f| ids.contains(&f.item.id.get()))
        .count();
    println!(
        "page one {} items, page two {} items, {overlap} shared",
        first.items.len(),
        second.items.len()
    );
    assert_eq!(overlap, 0, "the second page repeated items from the first");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn searching_by_text_finds_the_text() {
    let mut session = Session::anonymous().await.expect("session");
    let page = session
        .browse_workshop(&BrowseQuery {
            app: APP,
            text: Some("stargate".to_owned()),
            sort: BrowseSort::TextMatch,
            per_page: 10,
            ..BrowseQuery::default()
        })
        .await
        .expect("search");

    assert!(!page.items.is_empty(), "no results for a common term");
    let hits = page
        .items
        .iter()
        .filter(|found| {
            let haystack = format!("{} {}", found.item.title, found.description).to_lowercase();
            haystack.contains("stargate")
        })
        .count();
    println!("{hits} of {} results mention the term", page.items.len());
    assert!(
        hits > 0,
        "a text search matched nothing containing the text"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_search_result_downloads_without_a_second_lookup() {
    // The point of embedding WorkshopItem: what a search returns is what a
    // download takes. If this needs a GetDetails in between, the type is wrong.
    let root = std::env::var("TAPLINE_TEST_DIR")
        .unwrap_or_else(|_| format!("{}/.cache/tapline-test", std::env::var("HOME").unwrap()));
    let dir = std::path::PathBuf::from(root).join("browse-download");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");

    let mut session = Session::anonymous().await.expect("session");
    let page = session
        .browse_workshop(&BrowseQuery {
            app: APP,
            sort: BrowseSort::Subscribed,
            per_page: 20,
            ..BrowseQuery::default()
        })
        .await
        .expect("search");

    // Something small, so the test is about the handoff and not the bandwidth.
    let found = page
        .items
        .iter()
        .filter(|f| f.item.size > 0 && f.item.size < 8_000_000)
        .min_by_key(|f| f.item.size)
        .expect("a small item among the most-subscribed");

    println!(
        "downloading {} ({} bytes)",
        found.item.title, found.item.size
    );
    let options = tapline::InstallOptions {
        install_dir: dir.clone(),
        workshop_layout: tapline::WorkshopLayout::Flat,
        ..tapline::InstallOptions::default()
    };
    session
        .download_workshop_item(&found.item, &options)
        .await
        .expect("download straight from a search result");

    let written: Vec<_> = std::fs::read_dir(&dir)
        .expect("read")
        .filter_map(Result::ok)
        .collect();
    assert!(!written.is_empty(), "nothing was written");
    let _ = std::fs::remove_dir_all(&dir);
}
