//! An exploratory probe for the depot content path, not a gate.
//!
//! Gets a depot key, a CDN host list and a manifest request code, then fetches a
//! real manifest and dumps its first bytes — so the manifest parser can be
//! written against the format Steam actually serves rather than a description
//! of it.
//!
//! ```sh
//! cargo test -p tapline-rt-tokio --test explore_manifest -- --ignored --nocapture
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline_io::{Fetch, Request};
use tapline_net::{EMsg, Frame, Session};
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

/// TF2 Dedicated Server, and its smallest depot: 9,989 bytes installed.
const APP: u32 = 232_250;
const DEPOT: u32 = 232_257;

#[tokio::test]
#[ignore = "talks to Steam"]
async fn fetch_a_real_manifest_and_dump_it() {
    let servers = cm_list(0).await.expect("directory");
    let transport = CmTransport::connect(&servers.first().expect("a CM").endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    let outcome = session.logon_anonymous(0).await.expect("logon");
    println!("logged on, cell {}", outcome.cell_id);

    // --- the manifest id, from PICS -------------------------------------
    let info = tapline_pics::product_info(&mut session, tapline_ids::AppId(APP))
        .await
        .expect("PICS");
    let depot = info
        .depots(&tapline_pics::DepotFilter {
            os: tapline_pics::Os::Linux,
            branch: "public".to_owned(),
            include_dlc: false,
        })
        .into_iter()
        .find(|d| d.id.get() == DEPOT)
        .expect("the small depot");
    println!("depot {} manifest {}", depot.id, depot.manifest);

    // --- the depot key ---------------------------------------------------
    // Steam grants this only for content the session is entitled to, which is
    // the entitlement check tapline relies on rather than reimplements.
    let job = session.next_job_id();
    session
        .send(&Frame::new(
            EMsg::GET_DEPOT_DECRYPTION_KEY,
            header(&session, job),
            CMsgClientGetDepotDecryptionKey {
                depot_id: Some(DEPOT),
                app_id: Some(APP),
            }
            .encode_to_vec(),
        ))
        .await
        .expect("send depot key request");

    let reply = session.wait_for_job(job).await.expect("depot key response");
    let key_response: CMsgClientGetDepotDecryptionKeyResponse =
        reply.decode_body().expect("decode");
    println!(
        "depot key: eresult {:?}, {} bytes",
        key_response.eresult,
        key_response
            .depot_encryption_key
            .as_ref()
            .map_or(0, Vec::len)
    );
    assert_eq!(key_response.eresult, Some(1), "the depot key was refused");

    // --- the CDN host list -----------------------------------------------
    let servers_response = session
        .call(&CContentServerDirectory_GetServersForSteamPipe_Request {
            cell_id: Some(outcome.cell_id),
            max_servers: Some(8),
            ..CContentServerDirectory_GetServersForSteamPipe_Request::default()
        })
        .await
        .expect("GetServersForSteamPipe");

    println!("{} CDN servers offered", servers_response.servers.len());
    for server in servers_response.servers.iter().take(4) {
        println!(
            "  type {:?} host {:?} vhost {:?} https {:?} load {:?}",
            server.r#type, server.host, server.vhost, server.https_support, server.load
        );
    }
    let host = servers_response
        .servers
        .iter()
        .find(|s| s.host.is_some())
        .and_then(|s| s.host.clone())
        .expect("a CDN host");

    // --- the manifest request code ---------------------------------------
    // Without this, a modern manifest cannot be fetched at all.
    let code_response = session
        .call(&CContentServerDirectory_GetManifestRequestCode_Request {
            app_id: Some(APP),
            depot_id: Some(DEPOT),
            manifest_id: Some(depot.manifest.get()),
            app_branch: Some("public".to_owned()),
            branch_password_hash: None,
        })
        .await
        .expect("GetManifestRequestCode");
    let code = code_response
        .manifest_request_code
        .expect("a manifest request code");
    println!("manifest request code: {code}");

    // --- the manifest itself ---------------------------------------------
    let url = format!(
        "https://{host}/depot/{DEPOT}/manifest/{}/5/{code}",
        depot.manifest
    );
    println!("GET {url}");

    let client = HttpClient::new();
    let response = client
        .get(Request::get(&url), 32 * 1024 * 1024)
        .await
        .expect("the manifest fetch must succeed");

    println!("status {} — {} bytes", response.status, response.body.len());
    for (name, value) in &response.headers {
        println!("  {name}: {value}");
    }
    assert!(response.is_success(), "the CDN refused the manifest");

    println!(
        "first 128 bytes:\n{}",
        dump(response.body.get(..128).unwrap_or(&response.body))
    );

    let path = std::env::temp_dir().join(format!("manifest_{DEPOT}_{}.bin", depot.manifest));
    std::fs::write(&path, &response.body).expect("write the dump");
    println!("wrote {}", path.display());

    session.close().await.expect("close");
}

fn header(session: &Session<CmTransport>, job: u64) -> CMsgProtoBufHeader {
    CMsgProtoBufHeader {
        client_sessionid: Some(session.session_id()),
        steamid: Some(session.steam_id()),
        jobid_source: Some(job),
        ..CMsgProtoBufHeader::default()
    }
}

fn dump(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (index, chunk) in bytes.chunks(16).enumerate() {
        out.push_str(&format!("{:08x}  ", index * 16));
        for byte in chunk {
            out.push_str(&format!("{byte:02x} "));
        }
        for _ in chunk.len()..16 {
            out.push_str("   ");
        }
        out.push(' ');
        for byte in chunk {
            out.push(if byte.is_ascii_graphic() || *byte == b' ' {
                char::from(*byte)
            } else {
                '.'
            });
        }
        out.push('\n');
    }
    out
}
