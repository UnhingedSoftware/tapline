use tapline_net::Session;
use tapline_rt_tokio::{CmTransport, cm_list};

#[tokio::test(flavor = "multi_thread")]
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

    let loads: Vec<u32> = servers.iter().map(|s| s.load).collect();
    let mut sorted = loads.clone();
    sorted.sort_unstable();
    assert_eq!(loads, sorted, "the list must come back ordered by load");

    for server in &servers {
        assert!(
            server.endpoint.contains(':'),
            "endpoint {} has no port",
            server.endpoint
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_anonymous_logon_succeeds_against_a_real_cm() {
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

    assert_ne!(outcome.session_id, 0, "no session id assigned");
    assert_ne!(outcome.steam_id, 0, "no steamid assigned");

    let account_type = (outcome.steam_id >> 52) & 0xF;
    assert_eq!(account_type, 10, "expected an AnonUser steamid");
    assert_eq!(
        (outcome.steam_id >> 56) & 0xFF,
        1,
        "expected the public universe"
    );

    assert!(
        (1..=600).contains(&outcome.heartbeat_seconds),
        "implausible heartbeat interval {}",
        outcome.heartbeat_seconds
    );

    session.heartbeat().await.expect("heartbeat must send");

    session.close().await.expect("close must succeed");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn the_session_survives_the_traffic_steam_pushes_unasked() {
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
