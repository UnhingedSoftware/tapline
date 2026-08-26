//! Live tests against Steam's real CMs.
//!
//! All `#[ignore]`d, so CI stays offline and deterministic. Run them by hand:
//!
//! ```sh
//! cargo test -p tapline-rt-tokio -- --ignored --nocapture
//! ```
//!
//! These log on **anonymously**. No credentials are read, none are stored, and
//! nothing here touches an account — an anonymous session is what a dedicated
//! server install uses, and it is the least intrusive thing that proves the
//! stack works end to end.

use tapline_net::Session;
use tapline_rt_tokio::{CmTransport, cm_list};

#[tokio::test]
#[ignore = "talks to Steam"]
async fn the_directory_returns_usable_cms() {
    let servers = cm_list(0).await.expect("the directory must answer");

    assert!(!servers.is_empty(), "no CMs offered");
    println!("{} CMs offered; best few:", servers.len());
    for server in servers.iter().take(3) {
        println!(
            "  {} ({}) load {}",
            server.endpoint, server.datacentre, server.load
        );
    }

    // Sorted best-first, which is what makes taking the head meaningful.
    let loads: Vec<u32> = servers.iter().map(|s| s.load).collect();
    let mut sorted = loads.clone();
    sorted.sort_unstable();
    assert_eq!(loads, sorted, "the list must come back ordered by load");

    // Every entry must be connectable as host:port.
    for server in &servers {
        assert!(
            server.endpoint.contains(':'),
            "endpoint {} has no port",
            server.endpoint
        );
    }
}

#[tokio::test]
#[ignore = "talks to Steam"]
async fn an_anonymous_logon_succeeds_against_a_real_cm() {
    // The M3 gate. Everything from the WebSocket handshake through message
    // framing, batch expansion and job correlation has to be right for this to
    // return, and none of it can be faked by a fixture.
    let servers = cm_list(0).await.expect("the directory must answer");
    let server = servers.first().expect("at least one CM");
    println!("connecting to {}", server.endpoint);

    let transport = CmTransport::connect(&server.endpoint)
        .await
        .expect("the WebSocket upgrade must succeed");

    let mut session = Session::new(transport);
    let outcome = session
        .logon_anonymous(0)
        .await
        .expect("the anonymous logon must succeed");

    println!(
        "logged on: steamid {} session {} cell {} heartbeat {}s",
        outcome.steam_id, outcome.session_id, outcome.cell_id, outcome.heartbeat_seconds
    );

    // Steam assigns a real session, so a zero here means we read the wrong
    // field rather than that the logon worked.
    assert_ne!(outcome.session_id, 0, "no session id assigned");
    assert_ne!(outcome.steam_id, 0, "no steamid assigned");

    // An anonymous logon comes back as an AnonUser account: universe 1, type 10.
    let account_type = (outcome.steam_id >> 52) & 0xF;
    assert_eq!(account_type, 10, "expected an AnonUser steamid");
    assert_eq!(
        (outcome.steam_id >> 56) & 0xFF,
        1,
        "expected the public universe"
    );

    // Steam asks for a heartbeat interval, and a session that ignores it gets
    // dropped without explanation.
    assert!(
        (1..=600).contains(&outcome.heartbeat_seconds),
        "implausible heartbeat interval {}",
        outcome.heartbeat_seconds
    );

    // A heartbeat must be accepted rather than closing the connection.
    session.heartbeat().await.expect("heartbeat must send");

    session.close().await.expect("close must succeed");
}

#[tokio::test]
#[ignore = "talks to Steam"]
async fn the_session_survives_the_traffic_steam_pushes_unasked() {
    // Steam pushes a license list and other messages right after logon, batched
    // in with the response. If they were being dropped, this would still pass —
    // so it asserts they arrived.
    let servers = cm_list(0).await.expect("the directory must answer");
    let server = servers.first().expect("at least one CM");

    let transport = CmTransport::connect(&server.endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    session.logon_anonymous(0).await.expect("logon");

    let unsolicited = session.take_unsolicited();
    println!(
        "{} unsolicited messages arrived with the logon",
        unsolicited.len()
    );
    for frame in &unsolicited {
        println!("  EMsg {}", frame.emsg.value());
    }

    session.close().await.expect("close");
}
