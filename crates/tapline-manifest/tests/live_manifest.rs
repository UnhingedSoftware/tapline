//! The M5 gate, live: fetch a real manifest and decrypt its filenames.
//!
//! ```sh
//! cargo test -p tapline-manifest -- --ignored --nocapture
//! ```
//!
//! Anonymous session throughout. The depot key comes from Steam and is granted
//! only for content the session may have, which is the entitlement check tapline
//! relies on rather than reimplements.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline_ids::AppId;
use tapline_io::{Fetch, Request};
use tapline_manifest::Manifest;
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
use tapline_rt_tokio::{CmTransport, HttpClient, cm_list};
use tapline_wire::Message;

const APP: AppId = AppId(232_250);
/// The smallest depot in the app: 9,989 bytes installed.
const SMALL_DEPOT: u32 = 232_257;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn a_real_manifest_decrypts_into_real_paths() {
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
        .find(|d| d.id.get() == SMALL_DEPOT)
        .expect("the small depot");

    // --- depot key --------------------------------------------------------
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
                depot_id: Some(SMALL_DEPOT),
                app_id: Some(APP.get()),
            }
            .encode_to_vec(),
        ))
        .await
        .expect("send");
    let reply = session.wait_for_job(job).await.expect("depot key");
    let key_response: CMsgClientGetDepotDecryptionKeyResponse =
        reply.decode_body().expect("decode");
    assert_eq!(key_response.eresult, Some(1), "the depot key was refused");

    let key: [u8; 32] = key_response
        .depot_encryption_key
        .as_deref()
        .expect("a key")
        .try_into()
        .expect("a 32-byte AES key");

    // --- manifest ---------------------------------------------------------
    let cdn = session
        .call(&CContentServerDirectory_GetServersForSteamPipe_Request {
            cell_id: Some(outcome.cell_id),
            max_servers: Some(4),
            ..CContentServerDirectory_GetServersForSteamPipe_Request::default()
        })
        .await
        .expect("CDN list");
    let host = cdn
        .servers
        .iter()
        .find_map(|s| s.host.clone())
        .expect("a CDN host");

    let code = session
        .call(&CContentServerDirectory_GetManifestRequestCode_Request {
            app_id: Some(APP.get()),
            depot_id: Some(SMALL_DEPOT),
            manifest_id: Some(depot.manifest.get()),
            app_branch: Some("public".to_owned()),
            branch_password_hash: None,
        })
        .await
        .expect("manifest request code")
        .manifest_request_code
        .expect("a code");

    let url = format!(
        "https://{host}/depot/{SMALL_DEPOT}/manifest/{}/5/{code}",
        depot.manifest
    );
    let body = HttpClient::new()
        .get(Request::get(&url), 32 * 1024 * 1024)
        .await
        .expect("manifest fetch")
        .body;

    // --- the gate ---------------------------------------------------------
    let manifest = Manifest::parse(&body, Some(&key)).expect("the manifest must parse and decrypt");

    println!(
        "depot {} manifest {} — {} files, {} bytes, {} unique chunks",
        manifest.depot,
        manifest.id,
        manifest.files.len(),
        manifest.total_size,
        manifest.unique_chunks
    );
    for file in manifest.files.iter().take(20) {
        println!(
            "  {:>10}  {}{}",
            file.size,
            file.path,
            if file.flags.executable { " (+x)" } else { "" }
        );
    }

    assert_eq!(manifest.depot.get(), SMALL_DEPOT);
    assert_eq!(manifest.id, depot.manifest);
    assert!(!manifest.files.is_empty());

    // Decryption worked if the paths look like paths. A wrong key produces
    // either an error or bytes that are not UTF-8, so reaching here with
    // plausible names is the real assertion.
    let has_a_real_looking_path = manifest
        .files
        .iter()
        .any(|file| file.path.contains('/') || file.path.contains('.'));
    assert!(
        has_a_real_looking_path,
        "no filename decrypted into anything path-shaped: {:?}",
        manifest
            .files
            .iter()
            .map(|f| &f.path)
            .take(5)
            .collect::<Vec<_>>()
    );

    // Every path must be printable — a mis-decryption yields control bytes.
    for file in &manifest.files {
        assert!(
            file.path.chars().all(|c| !c.is_control()),
            "control characters in a decrypted path: {:?}",
            file.path
        );
    }

    // The sizes PICS and the manifest report must agree.
    assert_eq!(
        manifest.total_size, depot.size,
        "PICS and the manifest disagree about the depot's size"
    );

    let (chunks, download_bytes) = manifest.distinct_chunks();
    println!(
        "{} distinct chunks, {download_bytes} bytes to download",
        chunks.len()
    );
    assert!(!chunks.is_empty());
    for chunk in &chunks {
        assert_eq!(chunk.id.len(), 20, "a chunk id must be a SHA-1");
        assert!(chunk.compressed_size > 0);
    }

    session.close().await.expect("close");
}
