#![allow(clippy::expect_used, clippy::unwrap_used)]

use tapline_ids::AppId;
use tapline_net::Session;
use tapline_proto::steammessages_publishedfile_steamclient::{
    CPublishedFile_GetDetails_Request, CPublishedFile_QueryFiles_Request,
};
use tapline_rt_tokio::{CmTransport, cm_list};

const APPS: &[(u32, &str)] = &[
    (4000, "Garry's Mod"),
    (107_410, "Arma 3"),
    (730, "Counter-Strike 2"),
];

#[tokio::test(flavor = "multi_thread")]
#[ignore = "talks to Steam"]
async fn which_delivery_path_do_real_items_use() {
    let servers = cm_list(0).await.expect("directory");
    let transport = CmTransport::connect(&servers.first().expect("a CM").endpoint)
        .await
        .expect("connect");
    let mut session = Session::new(transport);
    session.logon_anonymous(0).await.expect("logon");

    let mut item_ids = Vec::new();
    for (appid, name) in APPS {
        let query = session
            .call(&CPublishedFile_QueryFiles_Request {
                query_type: Some(1),
                page: Some(1),
                numperpage: Some(4),
                appid: Some(*appid),
                return_details: Some(true),
                ..CPublishedFile_QueryFiles_Request::default()
            })
            .await
            .expect("QueryFiles");

        println!(
            "{name} ({appid}): {} items",
            query.publishedfiledetails.len()
        );
        for details in &query.publishedfiledetails {
            if let Some(id) = details.publishedfileid {
                println!(
                    "  {id}  {:?}  {} bytes",
                    details.title,
                    details.file_size.unwrap_or(0)
                );
                item_ids.push(id);
            }
        }
    }
    println!();

    let response = session
        .call(&CPublishedFile_GetDetails_Request {
            publishedfileids: item_ids.clone(),
            includetags: Some(false),
            includeadditionalpreviews: Some(false),
            includechildren: Some(true),
            includekvtags: Some(false),
            includevotes: Some(false),
            short_description: Some(true),
            ..CPublishedFile_GetDetails_Request::default()
        })
        .await
        .expect("PublishedFile.GetDetails");

    println!("{} items returned\n", response.publishedfiledetails.len());

    let mut steampipe = 0;
    let mut legacy = 0;
    let mut neither = 0;

    for details in &response.publishedfiledetails {
        println!("--- item {:?}", details.publishedfileid);
        println!("  result           {:?}", details.result);
        println!("  publishedfileid  {:?}", details.publishedfileid);
        println!("  title            {:?}", details.title);
        println!("  consumer_appid   {:?}", details.consumer_appid);
        println!("  creator_appid    {:?}", details.creator_appid);
        println!("  filename         {:?}", details.filename);
        println!("  file_size        {:?}", details.file_size);
        println!("  hcontent_file    {:?}", details.hcontent_file);
        println!("  file_url         {:?}", details.file_url);
        println!("  children         {}", details.children.len());

        let has_manifest = details.hcontent_file.is_some_and(|h| h != 0);
        let has_url = details.file_url.as_deref().is_some_and(|u| !u.is_empty());

        match (has_manifest, has_url) {
            (true, _) => {
                steampipe += 1;
                println!(
                    "  => SteamPipe UGC (manifest {})",
                    details.hcontent_file.unwrap_or(0)
                );
            }
            (false, true) => {
                legacy += 1;
                println!("  => legacy UFS blob");
            }
            (false, false) => {
                neither += 1;
                println!("  => neither: no manifest and no URL");
            }
        }
        println!();
    }

    println!("SteamPipe UGC: {steampipe}, legacy UFS: {legacy}, neither: {neither}");

    for app in [AppId(4000), AppId(730)] {
        match tapline_pics::product_info(&mut session, app).await {
            Ok(info) => println!(
                "app {app} ({:?}) workshop depot: {:?}",
                info.name(),
                info.workshop_depot()
            ),
            Err(error) => println!("app {app}: {error}"),
        }
    }

    session.close().await.expect("close");
}
