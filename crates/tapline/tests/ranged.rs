#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use tapline::Session;
use tapline_ids::PublishedFileId;

const ITEM: PublishedFileId = PublishedFileId(104_691_717);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn the_index_can_be_read_before_the_file_is_downloaded() {
    let mut session = Session::anonymous().await.expect("session");
    let details = session
        .workshop_details(&[ITEM])
        .await
        .expect("details")
        .into_iter()
        .next()
        .expect("one item")
        .expect("resolvable");

    let file = session
        .open_workshop_item(&details)
        .await
        .expect("open by range");
    println!("{} bytes in {} chunks", file.len(), file.chunk_count());

    let head = file.read(0, 64 * 1024).await.expect("read the head");
    let addon = tapline_gmad::parse_index(&head).expect("the index must parse");

    println!(
        "{:?}: {} entries, {} bytes unpacked — known after reading {} of {} bytes",
        addon.name,
        addon.entries.len(),
        addon.unpacked_size(),
        head.len(),
        file.len()
    );
    assert_eq!(addon.name, "PAC3");
    assert_eq!(addon.entries.len(), 348);
    assert!(
        (head.len() as u64) < file.len(),
        "the whole file was read to get the index"
    );

    let selected: Vec<(u64, u64)> = addon
        .entries
        .iter()
        .filter(|entry| tapline_gmad::glob_matches("lua/**", &entry.path))
        .map(|entry| (entry.offset as u64, entry.size))
        .collect();

    let everything = file.cost_of(&[(0, file.len())]);
    let only_lua = file.cost_of(&selected);
    println!(
        "{} of {} entries match lua/**: {} bytes to fetch against {} for the lot",
        selected.len(),
        addon.entries.len(),
        only_lua,
        everything
    );

    assert!(!selected.is_empty(), "the filter matched nothing");
    assert!(
        only_lua < everything,
        "a selective read should cost less than the whole file"
    );

    let pieces = file.read_many(&selected).await.expect("ranged read");
    assert_eq!(pieces.len(), selected.len());
    for (piece, (_, len)) in pieces.iter().zip(selected.iter()) {
        assert_eq!(piece.len() as u64, *len, "a range came back the wrong size");
    }

    let whole = file.read(0, file.len()).await.expect("read all");
    for (index, (offset, len)) in selected.iter().enumerate().take(5) {
        let expected = whole
            .get(*offset as usize..(*offset + *len) as usize)
            .expect("in range");
        assert_eq!(
            pieces.get(index).map(Vec::as_slice),
            Some(expected),
            "a ranged read returned different bytes than the whole file has"
        );
    }
    println!("ranged reads match the whole file");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn the_tail_can_be_read_first() {
    let mut session = Session::anonymous().await.expect("session");
    let details = session
        .workshop_details(&[ITEM])
        .await
        .expect("details")
        .into_iter()
        .next()
        .expect("one item")
        .expect("resolvable");

    let file = session.open_workshop_item(&details).await.expect("open");
    let tail_len = 4096.min(file.len());
    let tail = file
        .read(file.len() - tail_len, tail_len)
        .await
        .expect("read the tail");

    assert_eq!(tail.len() as u64, tail_len);

    let whole = file.read(0, file.len()).await.expect("read all");
    let expected = whole
        .get((file.len() - tail_len) as usize..)
        .expect("in range");
    assert_eq!(tail, expected, "the tail read returned the wrong bytes");
    println!(
        "read the last {tail_len} bytes of a {} byte file directly",
        file.len()
    );
}
