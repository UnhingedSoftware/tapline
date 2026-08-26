//! The M6 gate: download a real depot end to end and check what landed.
//!
//! ```sh
//! cargo test -p tapline-cdn -- --ignored --nocapture
//! ```
//!
//! Anonymous session throughout. Depot 232257 is 9,989 bytes across two chunks,
//! which is small enough to verify by eye and large enough to exercise every
//! stage: PICS, depot key, CDN directory, manifest request code, manifest fetch
//! and decrypt, chunk fetch, decrypt, decompress, hash-check, and the write.
//!
//! # Disk
//!
//! The install goes to `TAPLINE_TEST_DIR` (default `~/.cache/tapline-test`),
//! never `/tmp` — that is tmpfs on the development machine, and a larger depot
//! test there would be gigabytes of RAM. The directory removes itself
//! afterwards, including on panic.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;
use tapline_cdn::{Host, HostPool, fetch_chunk, fetch_manifest};
use tapline_fs::validate_path;
use tapline_ids::AppId;
use tapline_io::Sink;
use tapline_net::{EMsg, Frame, Session};
use tapline_pics::{DepotFilter, Os};
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_proto::steammessages_clientserver_2::{
    CMsgClientGetDepotDecryptionKey, CMsgClientGetDepotDecryptionKeyResponse,
};
use tapline_proto::steammessages_contentsystem_steamclient::{
    CContentServerDirectory_GetManifestRequestCode_Request,
    CContentServerDirectory_GetServersForSteamPipe_Request,
};
use tapline_rt_tokio::{CmTransport, FileSink, HttpClient, cm_list};
use tapline_wire::Message;

const APP: AppId = AppId(232_250);
const DEPOT: u32 = 232_257;

/// A scratch install directory that cleans up after itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
            |_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                    .join(".cache/tapline-test")
            },
            PathBuf::from,
        );
        assert!(
            !base.starts_with("/tmp"),
            "the scratch root must not be under /tmp, which is tmpfs here"
        );
        let path = base.join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("scratch");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Including on panic: a failed test must not leave an install behind.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
#[ignore = "talks to Steam"]
async fn a_real_depot_downloads_and_lands_on_disk() {
    let scratch = Scratch::new("install-232257");
    let root = &scratch.0;
    println!("installing into {}", root.display());

    // --- session, PICS, depot key ----------------------------------------
    let servers = cm_list(0).await.expect("directory");
    let transport = CmTransport::connect(&servers.first().expect("a CM").endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    let outcome = session.logon_anonymous(0).await.expect("logon");

    let info = tapline_pics::product_info(&mut session, APP)
        .await
        .expect("PICS");
    let depot = info
        .depots(&DepotFilter {
            os: Os::Linux,
            branch: "public".to_owned(),
            include_dlc: false,
        })
        .into_iter()
        .find(|d| d.id.get() == DEPOT)
        .expect("the small depot");

    let job = session.next_job_id();
    session
        .send(&Frame::new(
            EMsg::GET_DEPOT_DECRYPTION_KEY,
            CMsgProtoBufHeader {
                client_sessionid: Some(session.session_id()),
                steamid: Some(session.steam_id()),
                jobid_source: Some(job),
                ..CMsgProtoBufHeader::default()
            },
            CMsgClientGetDepotDecryptionKey {
                depot_id: Some(DEPOT),
                app_id: Some(APP.get()),
            }
            .encode_to_vec(),
        ))
        .await
        .expect("send");
    let key_response: CMsgClientGetDepotDecryptionKeyResponse = session
        .wait_for_job(job)
        .await
        .expect("key")
        .decode_body()
        .expect("decode");
    let key: [u8; 32] = key_response
        .depot_encryption_key
        .as_deref()
        .expect("a key")
        .try_into()
        .expect("32 bytes");

    // --- CDN pool ---------------------------------------------------------
    let directory = session
        .call(&CContentServerDirectory_GetServersForSteamPipe_Request {
            cell_id: Some(outcome.cell_id),
            max_servers: Some(8),
            ..CContentServerDirectory_GetServersForSteamPipe_Request::default()
        })
        .await
        .expect("CDN list");

    let hosts: Vec<Host> = directory
        .servers
        .iter()
        .filter_map(|server| {
            let host = server.host.clone()?;
            Some(Host {
                vhost: server.vhost.clone().unwrap_or_else(|| host.clone()),
                host,
                // Steam types this as int32; a negative would be nonsense, and
                // an unknown load must sort last rather than first.
                load: server
                    .load
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(u32::MAX),
                https_required: server.https_support.as_deref() == Some("mandatory"),
            })
        })
        .collect();
    println!("{} CDN hosts", hosts.len());
    assert!(!hosts.is_empty(), "no CDN hosts offered");
    let mut pool = HostPool::new(hosts);

    let code = session
        .call(&CContentServerDirectory_GetManifestRequestCode_Request {
            app_id: Some(APP.get()),
            depot_id: Some(DEPOT),
            manifest_id: Some(depot.manifest.get()),
            app_branch: Some("public".to_owned()),
            branch_password_hash: None,
        })
        .await
        .expect("code")
        .manifest_request_code
        .expect("a code");

    // --- manifest ---------------------------------------------------------
    let client = HttpClient::new();
    let host = pool.acquire().expect("a host");
    let manifest = fetch_manifest(
        &client,
        &host.host,
        depot.id,
        depot.manifest.get(),
        code,
        Some(&key),
    )
    .await
    .expect("the manifest must fetch and parse");

    println!(
        "manifest {} — {} files, {} bytes",
        manifest.id,
        manifest.files.len(),
        manifest.total_size
    );

    // --- the download -----------------------------------------------------
    let mut written_bytes = 0_u64;
    let mut written_files = Vec::new();

    for file in manifest.regular_files() {
        // Every path is validated before anything is opened. A Workshop
        // manifest's names are attacker-authored, and this is the check that
        // keeps them inside the install root.
        let safe = validate_path(&file.path)
            .unwrap_or_else(|e| panic!("the manifest named an unsafe path {:?}: {e}", file.path));
        let target = safe.resolve(root);

        let sink = FileSink::create(&target).expect("create the file");
        sink.allocate(file.size).await.expect("allocate");

        for chunk in &file.chunks {
            let host = pool.acquire().expect("a host");
            let plaintext = match fetch_chunk(&client, &host.host, depot.id, chunk, &key).await {
                Ok(bytes) => {
                    pool.succeed(&host.host);
                    bytes
                }
                Err(error) => {
                    pool.demote(&host.host);
                    panic!("chunk {} failed on {}: {error}", chunk.id_hex(), host.host);
                }
            };

            sink.write_at(chunk.offset, &plaintext)
                .await
                .expect("write");
            written_bytes += plaintext.len() as u64;
        }
        sink.sync().await.expect("sync");

        written_files.push((safe.as_str(), file.size, target));
    }

    // --- the gate ---------------------------------------------------------
    println!("wrote {} files, {written_bytes} bytes", written_files.len());
    for (path, size, target) in &written_files {
        let actual = std::fs::metadata(target)
            .expect("the file must exist")
            .len();
        println!("  {actual:>10}  {path}");
        assert_eq!(actual, *size, "{path} is the wrong size on disk");
    }

    assert_eq!(
        written_bytes, manifest.total_size,
        "the bytes written do not add up to the depot's size"
    );

    // The content itself, not just its length. These two files are what depot
    // 232257 contains, and their opening bytes are stable across builds.
    let whitelist = written_files
        .iter()
        .find(|(path, _, _)| path.ends_with("pure_server_whitelist.txt"))
        .map(|(_, _, target)| std::fs::read(target).expect("read"))
        .expect("the whitelist must have been installed");
    assert!(
        whitelist.starts_with(b"whitelist"),
        "the installed file is not what the depot contains"
    );

    let script = written_files
        .iter()
        .find(|(path, _, _)| path.ends_with(".py"))
        .map(|(_, _, target)| std::fs::read(target).expect("read"))
        .expect("the script must have been installed");
    assert!(script.starts_with(b"#!/usr/bin/env python3"));

    // Nothing may have been written outside the install root.
    for (path, _, target) in &written_files {
        assert!(
            target.starts_with(root),
            "{path} landed outside the install root at {target:?}"
        );
    }

    session.close().await.expect("close");
    println!("install verified; scratch directory will be removed");
}
