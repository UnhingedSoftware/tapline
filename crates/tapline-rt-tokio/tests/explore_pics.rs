//! An exploratory probe, not a gate.
//!
//! Dumps what a PICS product-info response actually contains so the parser can
//! be written against real bytes rather than against a guess. Kept because
//! re-running it is how the next surprise gets found.
//!
//! ```sh
//! cargo test -p tapline-rt-tokio --test explore_pics -- --ignored --nocapture
//! ```

use tapline_net::{EMsg, Frame, Session};
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_proto::steammessages_clientserver_appinfo::{
    CMsgClientPICSAccessTokenRequest, CMsgClientPICSAccessTokenResponse,
    CMsgClientPICSProductInfoRequest, CMsgClientPICSProductInfoResponse,
    c_msg_client_pics_product_info_request,
};
use tapline_rt_tokio::{CmTransport, cm_list};
use tapline_wire::Message;

/// Team Fortress 2 Dedicated Server: anonymous-accessible, and small.
const APP: u32 = 232_250;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn dump_a_real_pics_product_info_response() {
    let servers = cm_list(0).await.expect("directory");
    let server = servers.first().expect("a CM");
    let transport = CmTransport::connect(&server.endpoint)
        .await
        .expect("connect");

    let mut session = Session::new(transport);
    let outcome = session.logon_anonymous(0).await.expect("logon");
    println!("logged on, cell {}", outcome.cell_id);

    // Step one: an access token. Many apps refuse product info without it.
    let token_request = CMsgClientPICSAccessTokenRequest {
        appids: vec![APP],
        packageids: Vec::new(),
    };
    let job = 1000_u64;
    session
        .send(&Frame::new(
            EMsg::PICS_ACCESS_TOKEN_REQUEST,
            CMsgProtoBufHeader {
                client_sessionid: Some(session.session_id()),
                steamid: Some(session.steam_id()),
                jobid_source: Some(job),
                ..CMsgProtoBufHeader::default()
            },
            token_request.encode_to_vec(),
        ))
        .await
        .expect("send token request");

    let reply = session.wait_for_job(job).await.expect("token response");
    let tokens: CMsgClientPICSAccessTokenResponse = reply.decode_body().expect("decode");
    println!(
        "access tokens: {} granted, {} denied",
        tokens.app_access_tokens.len(),
        tokens.app_denied_tokens.len()
    );
    let token = tokens
        .app_access_tokens
        .first()
        .and_then(|t| t.access_token);
    println!("token for {APP}: {token:?}");

    // Step two: the product info itself.
    let info_request = CMsgClientPICSProductInfoRequest {
        apps: vec![c_msg_client_pics_product_info_request::AppInfo {
            appid: Some(APP),
            access_token: token,
            only_public_obsolete: None,
        }],
        packages: Vec::new(),
        meta_data_only: Some(false),
        ..CMsgClientPICSProductInfoRequest::default()
    };

    let job = 1001_u64;
    session
        .send(&Frame::new(
            EMsg::PICS_PRODUCT_INFO_REQUEST,
            CMsgProtoBufHeader {
                client_sessionid: Some(session.session_id()),
                steamid: Some(session.steam_id()),
                jobid_source: Some(job),
                ..CMsgProtoBufHeader::default()
            },
            info_request.encode_to_vec(),
        ))
        .await
        .expect("send product info request");

    let reply = session
        .wait_for_job(job)
        .await
        .expect("product info response");
    let info: CMsgClientPICSProductInfoResponse = reply.decode_body().expect("decode");

    println!(
        "apps: {}, unknown: {:?}, pending: {:?}, http_host: {:?}, http_min_size: {:?}",
        info.apps.len(),
        info.unknown_appids,
        info.response_pending,
        info.http_host,
        info.http_min_size
    );

    let app = info.apps.first().expect("one app");
    println!(
        "appid {:?} change {:?} missing_token {:?} size {:?} sha {:?}",
        app.appid,
        app.change_number,
        app.missing_token,
        app.size,
        app.sha.as_deref().map(hex)
    );

    let buffer = app.buffer.as_ref().expect("a buffer");
    println!("buffer: {} bytes", buffer.len());
    println!(
        "first 256 bytes:\n{}",
        dump(buffer.get(..256).unwrap_or(buffer))
    );
    println!(
        "last 64 bytes:\n{}",
        dump(
            buffer
                .get(buffer.len().saturating_sub(64)..)
                .unwrap_or(buffer)
        )
    );

    // Write it out so the parser can be developed and tested against it.
    let path = std::env::temp_dir().join(format!("pics_{APP}.bin"));
    std::fs::write(&path, buffer).expect("write the dump");
    println!("wrote {}", path.display());

    session.close().await.expect("close");
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A hex + ASCII dump, which is how the format's shape becomes visible.
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
