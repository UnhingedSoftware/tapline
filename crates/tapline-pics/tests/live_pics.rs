#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline_ids::AppId;
use tapline_net::Session;
use tapline_pics::{DepotFilter, Os, PicsError, product_info};
use tapline_rt_tokio::{CmTransport, cm_list};

const TF2_DS: AppId = AppId(232_250);
const CS2_DS: AppId = AppId(740);
const VALHEIM_DS: AppId = AppId(896_660);

async fn anonymous_session() -> Session<CmTransport> {
    let servers = cm_list(0).await.expect("the directory must answer");
    let server = servers.first().expect("at least one CM");
    let transport = CmTransport::connect(&server.endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    session.logon_anonymous(0).await.expect("anonymous logon");
    session
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn tf2_dedicated_server_resolves_to_real_depots() {
    let mut session = anonymous_session().await;
    let info = product_info(&mut session, TF2_DS)
        .await
        .expect("PICS must answer for an anonymous-accessible app");

    println!("{:?} — {:?}", info.name(), info.app_type());
    for branch in info.branches() {
        println!(
            "  branch {:<24} build {:?}{}",
            branch.name,
            branch.build_id,
            if branch.password_required {
                " (password)"
            } else {
                ""
            }
        );
    }

    let filter = DepotFilter {
        os: Os::Linux,
        branch: "public".to_owned(),
        include_dlc: false,
    };
    let depots = info.depots(&filter);
    for depot in &depots {
        println!(
            "  depot {:<8} manifest {:<20} {:>14} bytes ({} to download)",
            depot.id, depot.manifest, depot.size, depot.download_size
        );
    }
    println!("install size: {} bytes", info.install_size(&filter));

    assert_eq!(info.name(), Some("Team Fortress 2 Dedicated Server"));
    assert!(!depots.is_empty(), "a Linux install needs depots");

    assert!(
        depots.iter().all(|d| d.id.get() != 232_255),
        "the Windows-only depot was selected for a Linux install"
    );

    for depot in &depots {
        assert_ne!(
            depot.manifest.get(),
            0,
            "depot {} has no manifest",
            depot.id
        );
    }

    assert!(
        info.branches().iter().any(|b| b.name == "public"),
        "no public branch"
    );

    session.close().await.expect("close");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn the_windows_and_linux_filters_select_different_depots() {
    let mut session = anonymous_session().await;
    let info = product_info(&mut session, TF2_DS).await.expect("PICS");

    let linux: Vec<u32> = info
        .depots(&DepotFilter {
            os: Os::Linux,
            branch: "public".to_owned(),
            include_dlc: false,
        })
        .iter()
        .map(|d| d.id.get())
        .collect();
    let windows: Vec<u32> = info
        .depots(&DepotFilter {
            os: Os::Windows,
            branch: "public".to_owned(),
            include_dlc: false,
        })
        .iter()
        .map(|d| d.id.get())
        .collect();

    println!("linux:   {linux:?}");
    println!("windows: {windows:?}");
    assert_ne!(linux, windows, "the OS filter selected the same depots");

    session.close().await.expect("close");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn other_dedicated_servers_resolve_too() {
    let mut session = anonymous_session().await;

    for app in [CS2_DS, VALHEIM_DS] {
        match product_info(&mut session, app).await {
            Ok(info) => {
                let depots = info.depots(&DepotFilter {
                    os: Os::Linux,
                    branch: "public".to_owned(),
                    include_dlc: false,
                });
                println!(
                    "app {app}: {:?} — {} Linux depots, {} bytes",
                    info.name(),
                    depots.len(),
                    depots.iter().map(|d| d.size).sum::<u64>()
                );
                assert!(!depots.is_empty(), "app {app} resolved to no depots");
            }
            Err(PicsError::AccessDenied(_)) => {
                println!("app {app}: not available to an anonymous session");
            }
            Err(error) => panic!("app {app}: {error}"),
        }
    }

    session.close().await.expect("close");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn an_unknown_app_is_reported_as_unknown() {
    let mut session = anonymous_session().await;
    let result = product_info(&mut session, AppId(4_294_967_000)).await;
    println!("{result:?}");
    assert!(
        matches!(
            result,
            Err(PicsError::UnknownApp(_) | PicsError::AccessDenied(_))
        ),
        "an app that does not exist must not resolve"
    );
    session.close().await.expect("close");
}
