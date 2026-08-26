//! An exploratory probe for the chunk container, not a gate.
//!
//! Fetches one real chunk, decrypts it with the depot key, and dumps what is
//! inside — so the `VZ`/`VSZ` container can be implemented against the bytes
//! Steam actually serves. The headers around the compressed payload are
//! undocumented, which is the whole reason those two crates were deferred out of
//! M1: writing them earlier would have meant inventing a format and testing it
//! against my own invention.
//!
//! ```sh
//! cargo test -p tapline-rt-tokio --test explore_chunk -- --ignored --nocapture
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

/// Overridable so the probe can be pointed at a newer app, which is how the
/// question "does anything actually serve the zstd container?" gets answered.
fn target() -> (AppId, Option<u32>) {
    let app = std::env::var("TAPLINE_PROBE_APP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(232_250);
    let depot = std::env::var("TAPLINE_PROBE_DEPOT")
        .ok()
        .and_then(|v| v.parse().ok());
    (AppId(app), depot)
}

#[tokio::test]
#[ignore = "talks to Steam"]
async fn fetch_a_real_chunk_and_dump_its_container() {
    let servers = cm_list(0).await.expect("directory");
    let transport = CmTransport::connect(&servers.first().expect("a CM").endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    let outcome = session.logon_anonymous(0).await.expect("logon");

    let (app_id, wanted_depot) = target();
    let info = tapline_pics::product_info(&mut session, app_id)
        .await
        .expect("PICS");
    let mut candidates = info.depots(&DepotFilter {
        os: Os::Linux,
        branch: "public".to_owned(),
        include_dlc: false,
    });
    // Smallest first: one chunk is enough to read a container header, and the
    // smallest depot has the smallest manifest to fetch on the way there.
    candidates.sort_by_key(|d| d.size);
    let depot = match wanted_depot {
        Some(id) => candidates.into_iter().find(|d| d.id.get() == id),
        None => candidates.into_iter().next(),
    }
    .expect("a depot");
    let depot_id = depot.id.get();
    println!("app {app_id} depot {depot_id} — {} bytes", depot.size);

    // Depot key.
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
                depot_id: Some(depot_id),
                app_id: Some(app_id.get()),
            }
            .encode_to_vec(),
        ))
        .await
        .expect("send");
    let response: CMsgClientGetDepotDecryptionKeyResponse = session
        .wait_for_job(job)
        .await
        .expect("depot key")
        .decode_body()
        .expect("decode");
    let key: [u8; 32] = response
        .depot_encryption_key
        .as_deref()
        .expect("a key")
        .try_into()
        .expect("32 bytes");

    // CDN host and manifest.
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
        .expect("a host");

    let code = session
        .call(&CContentServerDirectory_GetManifestRequestCode_Request {
            app_id: Some(app_id.get()),
            depot_id: Some(depot_id),
            manifest_id: Some(depot.manifest.get()),
            app_branch: Some("public".to_owned()),
            branch_password_hash: None,
        })
        .await
        .expect("code")
        .manifest_request_code
        .expect("a code");

    let client = HttpClient::new();
    let manifest_bytes = client
        .get(
            Request::get(format!(
                "https://{host}/depot/{depot_id}/manifest/{}/5/{code}",
                depot.manifest
            )),
            32 * 1024 * 1024,
        )
        .await
        .expect("manifest")
        .body;
    let manifest = Manifest::parse(&manifest_bytes, Some(&key)).expect("manifest");

    // --- the chunks ------------------------------------------------------
    let (chunks, _) = manifest.distinct_chunks();
    println!("{} distinct chunks", chunks.len());

    for chunk in chunks.iter().take(3) {
        let url = format!("https://{host}/depot/{depot_id}/chunk/{}", chunk.id_hex());
        println!(
            "\nGET {url}\n  manifest says: {} compressed -> {} uncompressed, crc {:#010x}",
            chunk.compressed_size, chunk.uncompressed_size, chunk.crc
        );

        let response = client
            .get(Request::get(&url), 32 * 1024 * 1024)
            .await
            .expect("chunk fetch");
        println!(
            "  status {} — {} bytes",
            response.status,
            response.body.len()
        );
        assert!(response.is_success(), "the CDN refused the chunk");
        assert_eq!(
            response.body.len(),
            chunk.compressed_size as usize,
            "the CDN served a different length than the manifest promised"
        );

        println!(
            "  encrypted, first 32:\n{}",
            dump(response.body.get(..32).unwrap_or(&response.body))
        );

        // Content encryption: plain IV, no HMAC.
        let plain = tapline_crypto::decrypt_content(&key, &response.body)
            .expect("the chunk must decrypt with the depot key");
        println!("  decrypted to {} bytes", plain.len());
        println!(
            "  decrypted, first 64:\n{}",
            dump(plain.get(..64).unwrap_or(&plain))
        );
        println!(
            "  last 16:\n{}",
            dump(
                plain
                    .get(plain.len().saturating_sub(16)..)
                    .unwrap_or(&plain)
            )
        );

        // What container is it? The first two bytes are the tell.
        let magic = plain.get(..2).unwrap_or_default();
        println!(
            "  container magic: {magic:?} = {:?}",
            String::from_utf8_lossy(magic)
        );

        let path = std::env::temp_dir().join(format!("chunk_{}.bin", chunk.id_hex()));
        std::fs::write(&path, &plain).expect("write");
        println!("  wrote {} ({} bytes)", path.display(), plain.len());

        // And the encrypted form, since the pipeline has to handle that shape.
        let enc_path = std::env::temp_dir().join(format!("chunk_{}.enc", chunk.id_hex()));
        std::fs::write(&enc_path, &response.body).expect("write");
    }

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
