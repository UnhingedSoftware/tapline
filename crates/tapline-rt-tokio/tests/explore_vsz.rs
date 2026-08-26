//! Find and dump a `VSZ` chunk.
//!
//! Written after a full Valheim install failed with `expected a VZ container,
//! found magic "VS"`. An earlier probe sampled three chunks from the smallest
//! depot and saw `VZ` every time, which was a real measurement of an
//! unrepresentative sample — the conclusion drawn from it ("nothing serves the
//! zstd container") was wrong.
//!
//! This one scans until it finds one, so the answer comes from the depot rather
//! than from its first few chunks.
//!
//! ```sh
//! cargo test -p tapline-rt-tokio --test explore_vsz -- --ignored --nocapture
//! ```

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

/// Valheim Dedicated Server, whose install turned one up.
const APP: AppId = AppId(896_660);

#[tokio::test]
#[ignore = "talks to Steam"]
async fn find_a_vsz_chunk() {
    let servers = cm_list(0).await.expect("directory");
    let transport = CmTransport::connect(&servers.first().expect("a CM").endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    let outcome = session.logon_anonymous(0).await.expect("logon");

    let info = tapline_pics::product_info(&mut session, APP)
        .await
        .expect("PICS");
    let depots = info.depots(&DepotFilter {
        os: Os::Linux,
        branch: "public".to_owned(),
        include_dlc: false,
    });

    let cdn = session
        .call(&CContentServerDirectory_GetServersForSteamPipe_Request {
            cell_id: Some(outcome.cell_id),
            max_servers: Some(4),
            ..CContentServerDirectory_GetServersForSteamPipe_Request::default()
        })
        .await
        .expect("CDN");
    let host = cdn
        .servers
        .iter()
        .find_map(|s| s.host.clone())
        .expect("a host");
    let client = HttpClient::new();

    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut saved = false;

    for depot in depots {
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
                    depot_id: Some(depot.id.get()),
                    app_id: Some(depot.owner.get()),
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
        let Some(key_bytes) = key_response.depot_encryption_key.as_deref() else {
            continue;
        };
        let Ok(key) = <[u8; 32]>::try_from(key_bytes) else {
            continue;
        };

        let code = session
            .call(&CContentServerDirectory_GetManifestRequestCode_Request {
                app_id: Some(depot.owner.get()),
                depot_id: Some(depot.id.get()),
                manifest_id: Some(depot.manifest.get()),
                app_branch: Some("public".to_owned()),
                branch_password_hash: None,
            })
            .await
            .expect("code")
            .manifest_request_code
            .expect("a code");

        let body = client
            .get(
                Request::get(format!(
                    "https://{host}/depot/{}/manifest/{}/5/{code}",
                    depot.id, depot.manifest
                )),
                64 * 1024 * 1024,
            )
            .await
            .expect("manifest")
            .body;
        let manifest = Manifest::parse(&body, Some(&key)).expect("manifest");
        let (chunks, _) = manifest.distinct_chunks();
        println!("depot {} — {} chunks", depot.id, chunks.len());

        // Sample widely rather than taking the head: the head is what misled
        // the first probe.
        let step = (chunks.len() / 40).max(1);
        for chunk in chunks.iter().step_by(step).take(40) {
            let response = client
                .get(
                    Request::get(format!(
                        "https://{host}/depot/{}/chunk/{}",
                        depot.id,
                        chunk.id_hex()
                    )),
                    u64::from(chunk.compressed_size) + 4096,
                )
                .await
                .expect("chunk");

            let plain = tapline_crypto::decrypt_content(&key, &response.body).expect("decrypt");
            let magic = String::from_utf8_lossy(plain.get(..3).unwrap_or_default()).into_owned();
            *counts.entry(magic.clone()).or_insert(0) += 1;

            if magic.starts_with("VS") && !saved {
                saved = true;
                println!("\nFOUND a VSZ chunk: {}", chunk.id_hex());
                println!(
                    "  manifest says {} compressed -> {} uncompressed",
                    chunk.compressed_size, chunk.uncompressed_size
                );
                println!("  container is {} bytes", plain.len());
                println!("  first 64:\n{}", dump(plain.get(..64).unwrap_or(&plain)));
                println!(
                    "  last 32:\n{}",
                    dump(
                        plain
                            .get(plain.len().saturating_sub(32)..)
                            .unwrap_or(&plain)
                    )
                );
                let path = std::env::temp_dir().join(format!("vsz_{}.bin", chunk.id_hex()));
                std::fs::write(&path, &plain).expect("write");
                println!("  wrote {}", path.display());
            }
        }
        if saved {
            break;
        }
    }

    println!("\ncontainer magics seen: {counts:?}");
    session.close().await.expect("close");
}

fn dump(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (index, chunk) in bytes.chunks(16).enumerate() {
        out.push_str(&format!("    {:08x}  ", index * 16));
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
