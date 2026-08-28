#![allow(clippy::expect_used, clippy::unwrap_used)]

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
