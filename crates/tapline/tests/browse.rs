#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline::{AppId, BrowseQuery, BrowseSort, Session};

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

const TAGGED_APP: AppId = AppId(431_960);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn tag_groups_mean_any_within_a_group_and_all_across_them() {
    let mut session = Session::anonymous().await.expect("session");

    let mut count = async |query: BrowseQuery| -> u32 {
        session.browse_workshop(&query).await.expect("search").total
    };

    let base = BrowseQuery {
        app: TAGGED_APP,
        per_page: 1,
        ..BrowseQuery::default()
    };

    let scene_or_video_and_anime = count(BrowseQuery {
        tag_groups: vec![
            vec!["Scene".to_owned(), "Video".to_owned()],
            vec!["Anime".to_owned()],
        ],
        ..base.clone()
    })
    .await;
    let any_of_the_three = count(BrowseQuery {
        required_tags: vec!["Scene".to_owned(), "Video".to_owned(), "Anime".to_owned()],
        ..base.clone()
    })
    .await;
    let all_of_the_three = count(BrowseQuery {
        required_tags: vec!["Scene".to_owned(), "Video".to_owned(), "Anime".to_owned()],
        match_all_tags: true,
        ..base.clone()
    })
    .await;

    println!(
        "(Scene|Video)&Anime {scene_or_video_and_anime}, any {any_of_the_three}, all {all_of_the_three}"
    );

    assert!(
        scene_or_video_and_anime > all_of_the_three,
        "groups returned no more than requiring every tag"
    );
    assert!(
        scene_or_video_and_anime < any_of_the_three,
        "groups returned as much as accepting any tag"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn one_group_of_one_tag_is_the_same_search_as_a_required_tag() {
    let mut session = Session::anonymous().await.expect("session");
    let base = BrowseQuery {
        app: TAGGED_APP,
        per_page: 1,
        ..BrowseQuery::default()
    };

    let grouped = session
        .browse_workshop(&BrowseQuery {
            tag_groups: vec![vec!["Scene".to_owned()]],
            ..base.clone()
        })
        .await
        .expect("search")
        .total;
    let flat = session
        .browse_workshop(&BrowseQuery {
            required_tags: vec!["Scene".to_owned()],
            ..base
        })
        .await
        .expect("search")
        .total;

    println!("grouped {grouped}, flat {flat}");
    assert_eq!(grouped, flat, "one tag is one tag either way");
}
