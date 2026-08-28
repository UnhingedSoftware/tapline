#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;
use tapline::{AppId, InstallOptions, Os, PublishedFileId, Session, WorkshopContent};

const GMOD: AppId = AppId(4000);

fn scratch(name: &str) -> PathBuf {
    let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
        |_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                .join(".cache/tapline-test")
        },
        PathBuf::from,
    );
    assert!(
        !base.starts_with("/tmp"),
        "the scratch root must not be tmpfs"
    );
    let path = base.join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch");
    path
}

async fn some_real_items(session: &mut Session, app: AppId, count: u32) -> Vec<PublishedFileId> {
    use tapline_proto::steammessages_publishedfile_steamclient::CPublishedFile_QueryFiles_Request;

    let query = session
        .call_raw(&CPublishedFile_QueryFiles_Request {
            query_type: Some(1),
            page: Some(1),
            numperpage: Some(count),
            appid: Some(app.get()),
            return_details: Some(true),
            ..CPublishedFile_QueryFiles_Request::default()
        })
        .await
        .expect("QueryFiles");

    query
        .publishedfiledetails
        .iter()
        .filter_map(|details| details.publishedfileid.map(PublishedFileId))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_real_workshop_item_downloads_and_stays_inside_its_directory() {
    let root = scratch("workshop-gmod");
    let mut session = Session::anonymous().await.expect("session");

    let ids = some_real_items(&mut session, GMOD, 8).await;
    assert!(!ids.is_empty(), "Steam offered no Workshop items");
    println!("querying {} items", ids.len());

    let described = session
        .workshop_details(&ids)
        .await
        .expect("GetDetails must answer");

    let mut downloaded = 0;
    for outcome in &described {
        let item = match outcome {
            Ok(item) => item,
            Err(error) => {
                println!("  skipped: {error}");
                continue;
            }
        };

        println!(
            "  {} {:?} — {} bytes, {}",
            item.id,
            item.title,
            item.size,
            match &item.content {
                WorkshopContent::SteamPipe { depot, manifest } =>
                    format!("SteamPipe depot {depot} manifest {manifest}"),
                WorkshopContent::Legacy { url, .. } => format!("legacy {url}"),
            }
        );

        if item.size == 0 || item.size > 32 * 1024 * 1024 {
            continue;
        }

        let options = InstallOptions {
            install_dir: root.clone(),
            os: Os::Linux,
            ..InstallOptions::default()
        };

        match session.download_workshop_item(item, &options).await {
            Ok(report) => {
                println!(
                    "    downloaded {} files, {} bytes",
                    report.files, report.bytes_written
                );
                downloaded += 1;

                let item_dir = tapline::item_dir(&root, item.app, item.id);
                assert!(
                    item_dir.is_dir(),
                    "no directory was created for {}",
                    item.id
                );

                let mut files = 0;
                let mut stack = vec![item_dir.clone()];
                while let Some(dir) = stack.pop() {
                    for entry in std::fs::read_dir(&dir).expect("readable").flatten() {
                        let path = entry.path();
                        assert!(
                            path.starts_with(&item_dir),
                            "a Workshop item wrote outside its directory: {path:?}"
                        );
                        if path.is_dir() {
                            stack.push(path);
                        } else {
                            files += 1;
                        }
                    }
                }
                assert!(files > 0, "the item downloaded but wrote no files");
                println!(
                    "    {files} files on disk, all inside {}",
                    item_dir.display()
                );
            }
            Err(error) => {
                println!("    could not download: {error}");
            }
        }

        if downloaded >= 2 {
            break;
        }
    }

    assert!(
        downloaded > 0,
        "no Workshop item downloaded; every candidate was skipped or refused"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_nonexistent_item_is_reported_rather_than_silently_dropped() {
    let mut session = Session::anonymous().await.expect("session");
    let ids = vec![PublishedFileId(1), PublishedFileId(u64::MAX)];

    let described = session.workshop_details(&ids).await.expect("GetDetails");
    println!("{} outcomes for {} ids", described.len(), ids.len());
    for outcome in &described {
        println!("  {outcome:?}");
    }

    assert_eq!(
        described.len(),
        ids.len(),
        "the number of answers did not match the number of questions"
    );
    assert!(
        described.iter().all(std::result::Result::is_err),
        "a nonexistent item was described as downloadable"
    );
}

const DIFFERENTIAL_ITEM: PublishedFileId = PublishedFileId(3_790_437_566);

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam and needs a steamcmd workshop download to compare against"]
async fn tapline_downloads_what_steamcmd_downloads() {
    let Ok(reference_root) = std::env::var("TAPLINE_STEAMCMD_DIR") else {
        println!("SKIPPED: set TAPLINE_STEAMCMD_DIR to a steamcmd install directory");
        return;
    };
    let reference = PathBuf::from(reference_root)
        .join("steamapps/workshop/content/4000")
        .join(DIFFERENTIAL_ITEM.to_string());
    if !reference.is_dir() {
        println!("SKIPPED: {} does not exist", reference.display());
        return;
    }

    let root = scratch("workshop-differential");
    let mut session = Session::anonymous().await.expect("session");

    let described = session
        .workshop_details(&[DIFFERENTIAL_ITEM])
        .await
        .expect("GetDetails");
    let item = described
        .first()
        .expect("one answer")
        .as_ref()
        .expect("the item must be describable");

    let report = session
        .download_workshop_item(
            item,
            &InstallOptions {
                install_dir: root.clone(),
                os: Os::Linux,
                ..InstallOptions::default()
            },
        )
        .await
        .expect("download");
    println!(
        "tapline: {} files, {} bytes",
        report.files, report.bytes_written
    );

    let ours = tapline::item_dir(&root, item.app, item.id);

    let list = |dir: &std::path::Path| {
        let mut out: Vec<(String, Vec<u8>)> = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            for entry in std::fs::read_dir(&current).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if let (Ok(relative), Ok(bytes)) =
                    (path.strip_prefix(dir), std::fs::read(&path))
                {
                    out.push((relative.to_string_lossy().into_owned(), bytes));
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    };

    let theirs = list(&reference);
    let mine = list(&ours);

    println!(
        "steamcmd: {} files, tapline: {} files",
        theirs.len(),
        mine.len()
    );
    for (name, bytes) in &theirs {
        println!("  steamcmd {name}: {} bytes", bytes.len());
    }
    for (name, bytes) in &mine {
        println!("  tapline  {name}: {} bytes", bytes.len());
    }

    assert_eq!(
        theirs.len(),
        mine.len(),
        "the two tools produced a different number of files"
    );
    for ((their_name, their_bytes), (my_name, my_bytes)) in theirs.iter().zip(&mine) {
        assert_eq!(their_name, my_name, "the file names differ");
        assert_eq!(
            their_bytes, my_bytes,
            "{their_name} differs in content between the two tools"
        );
    }

    println!(
        "IDENTICAL: {} files, byte for byte, against steamcmd",
        theirs.len()
    );
    let _ = std::fs::remove_dir_all(&root);
}
