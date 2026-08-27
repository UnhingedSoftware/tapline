//! Identifying what a Workshop item is, without downloading it.
//!
//! ```sh
//! cargo test -p tapline-pipe --test probe -- --ignored --nocapture
//! ```
//!
//! A container is identified by its first few bytes, and a ranged read can
//! fetch exactly those. That is the cheapest possible use of the random-access
//! path and a good check that it works: if the offset arithmetic is wrong, four
//! bytes from offset zero is where it shows.

#![allow(clippy::expect_used, clippy::unwrap_used)]

/// A known Garry's Mod addon: 8.7 MB, and a GMAD.
const APP: u32 = 4_000;
const ITEM: u64 = 104_691_717;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn four_bytes_identify_a_container() {
    let mut session = tapline::Session::anonymous().await.expect("session");
    let details = session
        .workshop_details(&[tapline_ids::PublishedFileId(ITEM)])
        .await
        .expect("details");
    let item = details
        .into_iter()
        .next()
        .expect("one item")
        .expect("the item resolves");
    assert_eq!(item.app, tapline_ids::AppId(APP));

    let file = session.open_workshop_item(&item).await.expect("open");
    let head = file.read(0, 4).await.expect("read the magic");

    assert_eq!(&head, b"GMAD", "wrong magic for a Garry's Mod addon");

    // The point of the exercise: four bytes cost one chunk, not the file.
    let cost = file.cost_of(&[(0, 4)]);
    assert!(
        cost < file.len() / 4,
        "reading 4 bytes fetched {cost} of {} bytes",
        file.len()
    );
    println!(
        "{} bytes in {} chunks; a 4-byte read fetched {cost}",
        file.len(),
        file.chunk_count()
    );
}
